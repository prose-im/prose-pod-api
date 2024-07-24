// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod init;
pub mod invitations;
pub mod members;
pub mod server;
pub mod workspace;

use base64::{
    engine::general_purpose::{STANDARD as Base64, STANDARD_NO_PAD as Base64NoPad},
    Engine,
};
use cucumber::{given, then, when};
use prose_pod_api::{
    error::{self, Error},
    v1::LoginResponse,
};
use rocket::{
    http::{ContentType, Header},
    local::asynchronous::{Client, LocalResponse},
};
use secrecy::{ExposeSecret, SecretString};
use service::{
    model::MemberRole,
    prose_xmpp::{mods::AvatarData, BareJid},
    prosody_config::LuaValue,
    repositories::{MemberCreateForm, MemberRepository},
    services::jwt_service,
};

use crate::{cucumber_parameters::Duration, DbErr, TestWorld};

async fn name_to_jid(world: &TestWorld, name: &str) -> Result<BareJid, Error> {
    let domain = world.server_config().await?.domain;
    Ok(BareJid::new(&format!("{name}@{domain}")).map_err(|err| {
        error::InternalServerError(format!(
            "'{name}' cannot be used in a JID (or '{domain}' isn't a valid domain): {err}"
        ))
    })?)
}

#[given(expr = "{} is an admin")]
async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let member = MemberCreateForm {
        jid: jid.clone(),
        role: Some(MemberRole::Admin),
        joined_at: None,
    };
    let model = MemberRepository::create(db, member).await?;

    let token = world.mock_auth_service.log_in_unchecked(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

#[given(regex = r"^(.+) is (not an admin|a regular member|a member)$")]
async fn given_not_admin(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();

    let jid = name_to_jid(world, &name).await?;
    let model = world
        .user_service()
        .create_user(
            db,
            &jid,
            &SecretString::new("password".to_owned()),
            &name,
            &Some(MemberRole::Member),
        )
        .await?;

    let token = world.mock_auth_service.log_in_unchecked(&jid)?;

    world.members.insert(name, (model, token));

    Ok(())
}

#[given(regex = "^(\\w+) is (online|offline)$")]
async fn given_presence(
    world: &mut TestWorld,
    name: String,
    presence: String,
) -> Result<(), Error> {
    let mut state = world.xmpp_service_state_mut();

    let jid = name_to_jid(world, &name).await?;
    println!("{} is {}\n", name, presence);
    match presence.as_str() {
        "online" => state.online_members.insert(jid),
        "offline" => state.online_members.remove(&jid),
        p => panic!("Unexpected presence: '{p}'"),
    };

    Ok(())
}

#[given(expr = "{}'s avatar is {}")]
async fn given_avatar(world: &mut TestWorld, name: String, avatar: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world
        .mock_xmpp_service
        .set_avatar(&jid, Some(AvatarData::Base64(avatar)))?;
    Ok(())
}

#[given(expr = "{} has no avatar")]
async fn given_no_avatar(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    world.mock_xmpp_service.set_avatar(&jid, None)?;
    Ok(())
}

// LOGIN

async fn log_in<'a>(
    client: &'a Client,
    username: &BareJid,
    password: SecretString,
) -> LocalResponse<'a> {
    client
        .post("/v1/login")
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Basic {}", {
                let mut buf = String::new();
                Base64.encode_string(
                    format!("{}:{}", username, password.expose_secret()),
                    &mut buf,
                );
                buf
            }),
        ))
        .dispatch()
        .await
}

#[when(expr = "{} logs into the Prose Pod API")]
async fn when_user_logs_in(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    let password = world
        .mock_server_ctl
        .state
        .read()
        .unwrap()
        .users
        .get(&jid)
        .expect("User must be created first")
        .password
        .clone();
    let res = log_in(world.client(), &jid, password).await;
    world.result = Some(res.into());
    Ok(())
}

#[then(expr = "their access token should expire after {duration}")]
async fn then_token_expires_after(
    world: &mut TestWorld,
    duration: Duration,
) -> Result<(), jwt_service::Error> {
    let response: LoginResponse = world.result().body_into();
    let token: SecretString = response.token.expose_secret().clone().into();
    let (payload, _) = world.jwt_service.verify(&token)?;

    let issued_at = payload.issued_at().expect("JWT is missing 'Issued at'.");
    let expires_at = payload.expires_at().expect("JWT is missing 'Expires at'.");

    let lifetime = expires_at
        .duration_since(issued_at)
        .expect("Could not compute JWT lifetime.");
    assert_eq!(
        lifetime,
        std::time::Duration::from_secs(duration.seconds() as u64),
    );

    Ok(())
}

#[then(expr = "their access token shouldn't contain {string} when Base64-decoded")]
async fn then_token_encrypted(world: &mut TestWorld, pattern: String) -> Result<(), String> {
    let response: LoginResponse = world.result().body_into();
    let token: SecretString = response.token.expose_secret().clone().into();

    // Take only the JWT payload (JWT is formatted `<base64_json_header>.<base64_json_payload>.<binary_signature>`)
    let payload = token.expose_secret().split('.').skip(1).next().unwrap();
    assert!(!payload.contains(&pattern), "payload: {payload}");

    // Try to Base64-decode it
    let Ok(bytes) = Base64NoPad.decode(payload) else {
        // If if fails, then it means the token is encrypted
        return Ok(());
    };
    let decoded_token: String = match String::from_utf8(bytes) {
        Ok(decoded) => decoded,
        Err(err) => panic!(
            "Could not read the JWT as UTF-8: {err}\nToken: {}\nToken payload: {payload}",
            token.expose_secret(),
        ),
    };

    // Make sure the Base64-decoded JWT payload doesn't contain the given pattern
    assert!(
        !decoded_token.contains(&pattern),
        "decoded_token: {decoded_token}",
    );

    Ok(())
}

#[then(expr = "their Prosody access token should expire after {duration}")]
async fn then_prosody_token_expires_after(
    world: &mut TestWorld,
    duration: Duration,
) -> Result<(), DbErr> {
    let domain = world.server_config().await?.domain;

    let prosody_config = world
        .mock_server_ctl
        .state
        .read()
        .expect("`MockServerCtl` lock poisonned.")
        .applied_config
        .clone()
        .expect("XMPP server config not initialized");
    let settings = prosody_config
        .virtual_host_settings(&domain.to_string())
        .expect("Prosody config missing a `VirtualHost`.");

    assert_eq!(
        settings.custom_setting("oauth2_access_token_ttl"),
        Some(LuaValue::Number(duration.seconds().into())),
    );
    assert_eq!(
        settings.custom_setting("oauth2_refresh_token_ttl"),
        Some(LuaValue::Number(0.into())),
    );

    Ok(())
}
