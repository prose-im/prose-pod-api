// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_api::features::auth::ResetPasswordRequest;
use service::{
    auth::{auth_controller, password_reset_tokens, AuthServiceImpl},
    prosody_config::LuaValue,
    xmpp::ServerCtlImpl,
};

use super::prelude::*;

// MARK: - Given

#[given(regex = "^(\\w+) is (online|offline)$")]
async fn given_presence(
    world: &mut TestWorld,
    name: String,
    presence: String,
) -> Result<(), Error> {
    let mut state = world.xmpp_service_state_mut();

    let jid = name_to_jid(world, &name).await?;
    match presence.as_str() {
        "online" => state.online_members.insert(jid),
        "offline" => state.online_members.remove(&jid),
        p => panic!("Unexpected presence: '{p}'"),
    };

    Ok(())
}

#[given(expr = "{}’s password is {string}")]
async fn given_password(
    world: &mut TestWorld,
    name: String,
    password: String,
) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    (world.mock_server_ctl)
        .set_user_password(&jid, &password.into())
        .await?;

    Ok(())
}

#[given(expr = "{} requested a password reset for {}")]
async fn given_password_reset_requested(
    world: &mut TestWorld,
    actor: String,
    subject: String,
) -> Result<(), Error> {
    let actor_jid = name_to_jid(world, &actor).await?;
    let subject_jid = name_to_jid(world, &subject).await?;

    let token = world.mock_auth_service.log_in_unchecked(&actor_jid).await?;
    let user_info = (world.mock_auth_service)
        .get_user_info(token, &world.db.read)
        .await?;

    auth_controller::request_password_reset(
        &world.db.write,
        &world.notifcation_service(),
        &world.app_config(),
        &user_info,
        &subject_jid,
    )
    .await?;

    let tokens = password_reset_tokens::get_by_jid(&world.db.read, &subject_jid).await?;
    world.password_reset_tokens.insert(subject_jid, tokens);

    Ok(())
}

// MARK: - When

async fn log_in(api: &TestServer, username: &BareJid, password: SecretString) -> TestResponse {
    api.post("/v1/login")
        .add_header(CONTENT_TYPE, "application/json")
        .add_header(
            AUTHORIZATION,
            format!("Basic {}", {
                let mut buf = String::new();
                BASE64_STANDARD.encode_string(
                    format!("{}:{}", username, password.expose_secret()),
                    &mut buf,
                );
                buf
            }),
        )
        .await
}

#[when(expr = "{} logs into the Prose Pod API")]
async fn when_user_logs_in(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    let password = {
        let ref users = world.server_ctl_state().users;
        let user = users.get(&jid).expect("User must be created first");
        user.password.clone()
    };
    let res = log_in(world.api(), &jid, password).await;
    world.result = Some(res.into());
    Ok(())
}

api_call_fn!(
    request_password_reset,
    DELETE,
    "/v1/members/{jid}/password"; jid=BareJid,
);

#[when(expr = "{} requests a password reset for {}")]
async fn when_password_reset_request(
    world: &mut TestWorld,
    actor: String,
    subject: String,
) -> Result<(), Error> {
    let actor_token = user_token!(world, actor);
    let subject_jid = name_to_jid(world, &subject).await?;

    let res = request_password_reset(world.api(), actor_token, subject_jid.clone()).await;
    world.result = Some(res.unwrap().into());

    let tokens = password_reset_tokens::get_by_jid(&world.db.read, &subject_jid).await?;
    world.password_reset_tokens.insert(subject_jid, tokens);

    Ok(())
}

api_call_fn!(
    reset_password,
    unauthenticated: PUT,
    "/v1/password-reset-tokens/{token}/use"; token=String,
    payload: ResetPasswordRequest,
);

async fn when_password_reset_n(
    world: &mut TestWorld,
    name: String,
    password: String,
    n: usize,
) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;

    let tokens = (world.password_reset_tokens.get(&jid))
        .expect("A password reset request must be sent first.");
    let token = tokens.get(n).expect(&format!(
        "{n} password reset request(s) must be sent first.",
        n = n + 1
    ));

    use secrecy::ExposeSecret;
    let res = reset_password(
        world.api(),
        token.expose_secret().to_owned(),
        ResetPasswordRequest {
            password: password.into(),
        },
    )
    .await;
    world.result = Some(res.unwrap().into());
    Ok(())
}

#[when(expr = "an unauthenticated user uses {}’s password reset token with password {string}")]
async fn when_password_reset(
    world: &mut TestWorld,
    name: String,
    password: String,
) -> Result<(), Error> {
    when_password_reset_n(world, name, password, 0).await
}

#[when(
    expr = "an unauthenticated user uses {}’s first password reset token with password {string}"
)]
async fn when_password_reset_1(
    world: &mut TestWorld,
    name: String,
    password: String,
) -> Result<(), Error> {
    when_password_reset_n(world, name, password, 0).await
}

#[when(
    expr = "an unauthenticated user uses {}’s second password reset token with password {string}"
)]
async fn when_password_reset_2(
    world: &mut TestWorld,
    name: String,
    password: String,
) -> Result<(), Error> {
    when_password_reset_n(world, name, password, 1).await
}

// MARK: - Then

#[then(expr = "<{jid}>'s password is changed")]
fn then_password_changed(world: &mut TestWorld, jid: parameters::JID) {
    assert_ne!(world.mock_secrets_store().changes_count(&jid), 0);
}

#[then(expr = "their Prosody access token should expire after {duration}")]
async fn then_prosody_token_expires_after(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> Result<(), DbErr> {
    let domain = world.app_config().server_domain().clone();

    let server_ctl_state = world.server_ctl_state();
    let prosody_config = (server_ctl_state.applied_config)
        .as_ref()
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

#[then(expr = "there should be {int} valid password reset token(s) for {}")]
async fn then_n_valid_password_reset_tokens(
    world: &mut TestWorld,
    n: usize,
    name: String,
) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    let entries = password_reset_tokens::get_by_jid(&world.db.read, &jid).await?;
    assert_eq!(entries.len(), n);
    Ok(())
}

#[then(expr = "{}’s password should be {string}")]
async fn then_password(world: &mut TestWorld, name: String, expected: String) -> Result<(), Error> {
    let jid = name_to_jid(world, &name).await?;
    let password = {
        let ref users = world.server_ctl_state().users;
        let user = users.get(&jid).expect("User must be created first");
        user.password.clone()
    };
    assert_eq!(password.expose_secret(), expected);
    Ok(())
}
