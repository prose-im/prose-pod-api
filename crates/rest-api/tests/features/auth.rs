// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::prosody_config::LuaValue;

use super::prelude::*;

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

async fn log_in(api: &TestServer, username: &BareJid, password: SecretString) -> TestResponse {
    api.post("/v1/login")
        .add_header(CONTENT_TYPE, "application/json")
        .add_header(
            AUTHORIZATION,
            format!("Basic {}", {
                let mut buf = String::new();
                Base64.encode_string(
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
    let res = log_in(world.api(), &jid, password).await;
    world.result = Some(res.into());
    Ok(())
}

#[then(expr = "their Prosody access token should expire after {duration}")]
async fn then_prosody_token_expires_after(
    world: &mut TestWorld,
    duration: parameters::Duration,
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
