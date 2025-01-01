// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod dns_setup;
pub mod init;
pub mod invitations;
pub mod members;
pub mod network_checks;
pub mod pod_config;
pub mod roles;
pub mod server_config;
pub mod workspace_details;

use std::str::FromStr;

use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use cucumber::{given, then, when};
use prose_pod_api::error::{self, Error};
use rocket::{
    http::{ContentType, Header},
    local::asynchronous::{Client, LocalResponse},
};
use secrecy::{ExposeSecret, SecretString};
use service::{
    models::xmpp::{AvatarData, BareJid},
    prosody_config::LuaValue,
};

use crate::{cucumber_parameters::Duration, DbErr, TestWorld};

async fn name_to_jid(world: &TestWorld, name: &str) -> Result<BareJid, Error> {
    // Strip potential `<>` around the JID (if `name` is a JID).
    let name = name
        .strip_prefix("<")
        .and_then(|name| name.strip_suffix(">"))
        .unwrap_or(name);
    // Use JID if it already is one.
    if name.contains("@") {
        if let Ok(jid) = BareJid::from_str(name) {
            return Ok(jid);
        }
    }

    let domain = world.server_config().await?.domain;
    Ok(BareJid::new(&format!("{name}@{domain}")).map_err(|err| {
        error::InternalServerError(format!(
            "'{name}' cannot be used in a JID (or '{domain}' isn't a valid domain): {err}"
        ))
    })?)
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
