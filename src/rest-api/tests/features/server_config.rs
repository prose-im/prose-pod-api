// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    models::{DateLike, Duration, PossiblyInfinite},
    prosody::IntoProsody as _,
    prosody_config::linked_hash_set::LinkedHashSet,
    prosody_config_from_db,
    sea_orm::{ActiveModelTrait as _, EntityTrait, IntoActiveModel as _, ModelTrait, Set},
    server_config::entities::server_config,
};

use super::prelude::*;

async fn given_server_config(
    world: &mut TestWorld,
    update: impl FnOnce(
        &mut <<server_config::Model as ModelTrait>::Entity as EntityTrait>::ActiveModel,
    ) -> (),
) -> Result<(), Error> {
    let app_config = &world.app_config;

    let mut server_config = world.server_config_model().await?.into_active_model();
    update(&mut server_config);
    let model = server_config.update(world.db()).await?;
    let server_config = model.with_default_values_from(app_config);

    world.server_ctl_state_mut().applied_config =
        Some(prosody_config_from_db(server_config, app_config));

    Ok(())
}

#[then("the server should have been reconfigured")]
fn then_server_reconfigured(world: &mut TestWorld) {
    assert_ne!(world.server_ctl_state().conf_reload_count, 0);
}

#[then("the server should not have been reconfigured")]
fn then_server_not_reconfigured(world: &mut TestWorld) {
    assert_eq!(world.server_ctl_state().conf_reload_count, 0);
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
) -> Result<(), Error> {
    given_server_config(world, |model| {
        model.message_archive_enabled = Set(Some(state.into()));
    })
    .await
}

#[given(expr = "the message archive retention is set to {duration}")]
async fn given_message_archive_retention(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> Result<(), Error> {
    given_server_config(world, |model| {
        model.message_archive_retention = Set(Some(duration.into()));
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
    world.result = Some(res.into());
}

#[when(expr = "{} sets the message archive retention to {duration}")]
async fn when_set_message_archive_retention(
    world: &mut TestWorld,
    name: String,
    duration: parameters::Duration,
) {
    let token = user_token!(world, name);
    let res = set_message_archive_retention(world.api(), token, duration.into()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} resets the Messaging configuration to its default value")]
async fn when_reset_messaging_configuration(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = reset_messaging_configuration(world.api(), token).await;
    world.result = Some(res.into());
}

#[when(expr = "{} resets the message archive retention to its default value")]
async fn when_reset_message_archive_retention(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = reset_message_archive_retention(world.api(), token).await;
    world.result = Some(res.into());
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
) -> Result<(), DbErr> {
    let duration = duration.into();

    // Check in database
    let server_config = world.server_config().await?;
    assert_eq!(server_config.message_archive_retention, duration);

    // Check applied Prosody configuration
    let prosody_config = world.server_ctl_state().applied_config.unwrap();
    let global_settings = prosody_config.global_settings.to_owned();
    assert_eq!(
        global_settings.archive_expires_after,
        Some(duration.into_prosody())
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
) -> Result<(), DbErr> {
    let mut server_config = world.server_config_model().await?.into_active_model();
    server_config.file_upload_allowed = Set(Some(state.into()));
    server_config.update(world.db()).await?;
    Ok(())
}

#[given(expr = "the file retention is set to {duration}")]
async fn given_file_retention(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> Result<(), DbErr> {
    let mut server_config = world.server_config_model().await?.into_active_model();
    server_config.file_storage_retention = Set(Some(duration.into()));
    server_config.update(world.db()).await?;
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
    world.result = Some(res.into());
}

#[when(expr = "{} resets the Files configuration to its default value")]
async fn when_reset_files_configuration(world: &mut TestWorld, name: String) {
    let token = user_token!(world, name);
    let res = reset_files_configuration(world.api(), token).await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the file retention to {duration}")]
async fn when_set_file_retention(
    world: &mut TestWorld,
    name: String,
    duration: parameters::Duration,
) {
    let token = user_token!(world, name);
    let res = set_file_retention(world.api(), token, duration.into()).await;
    world.result = Some(res.into());
}

#[then(expr = "file uploading should be {toggle}")]
async fn then_file_uploading(
    world: &mut TestWorld,
    state: parameters::ToggleState,
) -> Result<(), DbErr> {
    let server_config = world.server_config().await?;

    assert_eq!(server_config.file_upload_allowed, state.as_bool());

    Ok(())
}

#[then(expr = "the file retention should be set to {duration}")]
async fn then_file_retention(
    world: &mut TestWorld,
    duration: parameters::Duration,
) -> Result<(), DbErr> {
    let server_config = world.server_config().await?;

    assert_eq!(server_config.file_storage_retention, duration.into());

    Ok(())
}
