// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

use cucumber::gherkin::Step;
use service::{
    global_storage,
    models::{DateLike, Duration, PossiblyInfinite},
    prosody::IntoProsody as _,
    prosody_config::linked_hash_set::LinkedHashSet,
    server_config,
};

use super::prelude::*;

#[given(regex = "the server config is")]
async fn given_server_config_in_db(world: &mut TestWorld, step: &Step) -> anyhow::Result<()> {
    let json = step.docstring().unwrap().trim();
    let map = serde_json::from_str(json)?;
    global_storage::kv_store::set_map(world.db(), "server_config", map).await?;
    Ok(())
}

api_call_fn!(get_server_config, GET, "/v1/server/config");

#[when(expr = "{} queries the server configuration")]
async fn when_get_server_config(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = get_server_config(world.api(), token).await;
    world.result = Some(res.unwrap().into());
}

#[then("the server should have been reconfigured")]
fn then_server_reconfigured(world: &mut TestWorld) {
    assert_ne!(world.server_ctl_state().conf_reload_count, 0);
}

#[then("the server should not have been reconfigured")]
fn then_server_not_reconfigured(world: &mut TestWorld) {
    assert_eq!(world.server_ctl_state().conf_reload_count, 0);
}

async fn given_server_config<F, R>(
    world: &mut TestWorld,
    update: impl FnOnce(DatabaseConnection) -> F,
) -> anyhow::Result<()>
where
    F: Future<Output = anyhow::Result<R>> + 'static,
{
    update(world.db.clone()).await?;
    world.server_config_manager().reload().await?;
    world.reset_server_ctl_counts();
    Ok(())
}

// MESSAGE ARCHIVING

api_call_fn!(
    reset_messaging_configuration,
    DELETE,
    "/v1/server/config/messaging"
);
api_call_fn!(
    set_message_archiving,
    PUT,
    "/v1/server/config/message-archive-enabled",
    payload: bool,
);
api_call_fn!(
    set_message_archive_retention,
    PUT,
    "/v1/server/config/message-archive-retention",
    payload: PossiblyInfinite<Duration<DateLike>>,
);
api_call_fn!(
    reset_message_archive_retention,
    DELETE,
    "/v1/server/config/message-archive-retention"
);

#[given(expr = "message archiving is {toggle}")]
async fn given_message_archiving(
    world: &mut TestWorld,
    state: parameters::ToggleState,
) -> anyhow::Result<()> {
    given_server_config(world, |db| async move {
        server_config::message_archive_enabled::set(&db, state.into()).await
    })
    .await
}

#[given(expr = "the message archive retention is set to {duration}")]
async fn given_message_archive_retention(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> anyhow::Result<()> {
    given_server_config(world, |db| async move {
        server_config::message_archive_retention::set(&db, duration.into()).await
    })
    .await
}

#[when(expr = "{} turns message archiving {toggle}")]
async fn when_set_message_archiving(
    world: &mut TestWorld,
    name: String,
    state: parameters::ToggleState,
) {
    let token = user_token!(world, name);
    let res = set_message_archiving(world.api(), token, state.into()).await;
    world.result = Some(res.unwrap().into());
}

#[when(expr = "{} sets the message archive retention to {duration}")]
async fn when_set_message_archive_retention(
    world: &mut TestWorld,
    name: String,
    duration: parameters::Duration,
) {
    let token = user_token!(world, name);
    let res = set_message_archive_retention(world.api(), token, duration.into()).await;
    world.result = Some(res.unwrap().into());
}

#[when(expr = "{} resets the Messaging configuration to its default value")]
async fn when_reset_messaging_configuration(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = reset_messaging_configuration(world.api(), token).await;
    world.result = Some(res.unwrap().into());
}

#[when(expr = "{} resets the message archive retention to its default value")]
async fn when_reset_message_archive_retention(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = reset_message_archive_retention(world.api(), token).await;
    world.result = Some(res.unwrap().into());
}

#[then(expr = "message archiving should be {toggle}")]
async fn then_message_archiving(
    world: &mut TestWorld,
    enabled: parameters::ToggleState,
) -> Result<(), Error> {
    let enabled = enabled.as_bool();

    // Check in database
    let server_config = world.server_config().await?;
    assert_eq!(server_config.message_archive_enabled, enabled);

    // Check applied Prosody configuration
    let prosody_config = world.server_ctl_state().applied_config.unwrap();
    let global_settings = prosody_config.global_settings.to_owned();
    let muc_settings = prosody_config
        .component_settings("muc")
        .expect(r#"The "muc" component should exist"#)
        .to_owned();
    let global_modules = global_settings.modules_enabled.unwrap_or_default();
    let muc_modules = muc_settings.modules_enabled.unwrap_or_default();
    assert_contains_if(enabled, &global_modules, "mam", LinkedHashSet::contains);
    assert_defined_if(enabled, global_settings.archive_expires_after);
    assert_defined_if(enabled, global_settings.default_archive_policy);
    assert_defined_if(enabled, global_settings.max_archive_query_results);
    assert_contains_if(enabled, &muc_modules, "muc_mam", LinkedHashSet::contains);

    Ok(())
}

#[then(expr = "the message archive retention should be set to {duration}")]
async fn then_message_archive_retention(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> anyhow::Result<()> {
    let duration = duration.into();

    // Check in database
    let server_config = world.server_config().await?;
    assert_eq!(server_config.message_archive_retention, duration, "db");

    // Check applied Prosody configuration
    let prosody_config = world.server_ctl_state().applied_config.unwrap();
    let global_settings = prosody_config.global_settings.to_owned();
    assert_eq!(
        global_settings.archive_expires_after,
        Some(duration.into_prosody()),
        "prosody",
    );

    Ok(())
}

// FILE UPLOADING

api_call_fn!(reset_files_configuration, DELETE, "/v1/server/config/files");
api_call_fn!(
    set_file_uploading,
    PUT,
    "/v1/server/config/file-upload-allowed",
    payload: bool,
);
api_call_fn!(
    set_file_retention,
    PUT,
    "/v1/server/config/file-storage-retention",
    payload: PossiblyInfinite<Duration<DateLike>>,
);

#[given(expr = "file uploading is {toggle}")]
async fn given_file_uploading(
    world: &mut TestWorld,
    state: parameters::ToggleState,
) -> anyhow::Result<()> {
    server_config::file_upload_allowed::set(world.db(), state.into()).await?;
    Ok(())
}

#[given(expr = "the file retention is set to {duration}")]
async fn given_file_retention(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> anyhow::Result<()> {
    server_config::file_storage_retention::set(world.db(), duration.into()).await?;
    Ok(())
}

#[when(expr = "{} turns file uploading {toggle}")]
async fn when_set_file_uploading(
    world: &mut TestWorld,
    name: String,
    state: parameters::ToggleState,
) {
    let token = user_token!(world, name);
    let res = set_file_uploading(world.api(), token, state.into()).await;
    world.result = Some(res.unwrap().into());
}

#[when(expr = "{} resets the Files configuration to its default value")]
async fn when_reset_files_configuration(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = reset_files_configuration(world.api(), token).await;
    world.result = Some(res.unwrap().into());
}

#[when(expr = "{} sets the file retention to {duration}")]
async fn when_set_file_retention(
    world: &mut TestWorld,
    name: String,
    duration: parameters::Duration,
) {
    let token = user_token!(world, name);
    let res = set_file_retention(world.api(), token, duration.into()).await;
    world.result = Some(res.unwrap().into());
}

#[then(expr = "file uploading should be {toggle}")]
async fn then_file_uploading(
    world: &mut TestWorld,
    state: parameters::ToggleState,
) -> anyhow::Result<()> {
    let server_config = world.server_config().await?;

    assert_eq!(server_config.file_upload_allowed, state.as_bool());

    Ok(())
}

#[then(expr = "the file retention should be set to {duration}")]
async fn then_file_retention(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> anyhow::Result<()> {
    let server_config = world.server_config().await?;

    assert_eq!(server_config.file_storage_retention, duration.into());

    Ok(())
}
