// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::{given, then, when};
use migration::DbErr;
use prose_pod_api::{error::Error, v1::server::config::*};
use rocket::http::{ContentType, Header};
use rocket::local::asynchronous::{Client, LocalResponse};
use secrecy::{ExposeSecret as _, SecretString};
use serde_json::json;
use service::prosody::IntoProsody as _;
use service::prosody_config::linked_hash_set::LinkedHashSet;
use service::repositories::ServerConfigRepository;
use service::sea_orm::{ActiveModelTrait as _, IntoActiveModel as _, Set};

use crate::cucumber_parameters::{Duration, ToggleState};
use crate::util::*;
use crate::v1::server::given_server_config;
use crate::TestWorld;

// MESSAGE ARCHIVING

async fn set_message_archiving<'a>(
    client: &'a Client,
    token: SecretString,
    state: bool,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/config/store-message-archive")
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .body(
            json!(SetMessageArchivingRequest {
                message_archive_enabled: state,
            })
            .to_string(),
        )
        .dispatch()
        .await
}

async fn set_message_archive_retention<'a>(
    client: &'a Client,
    token: SecretString,
    duration: Duration,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/config/message-archive-retention")
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .body(
            json!(SetMessageArchiveRetentionRequest {
                message_archive_retention: duration.into(),
            })
            .to_string(),
        )
        .dispatch()
        .await
}

async fn reset_reset_messaging_confiuration<'a>(
    client: &'a Client,
    token: SecretString,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/config/messaging/reset")
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .dispatch()
        .await
}

async fn reset_message_archive_retention<'a>(
    client: &'a Client,
    token: SecretString,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/config/message-archive-retention/reset")
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .dispatch()
        .await
}

#[given(expr = "message archiving is {toggle}")]
async fn given_message_archiving(world: &mut TestWorld, state: ToggleState) -> Result<(), Error> {
    given_server_config(world, |model| {
        model.message_archive_enabled = Set(Some(state.into()));
    })
    .await
}

#[given(expr = "the message archive retention is set to {duration}")]
async fn given_message_archive_retention(
    world: &mut TestWorld,
    duration: Duration,
) -> Result<(), Error> {
    given_server_config(world, |model| {
        model.message_archive_retention = Set(Some(duration.into()));
    })
    .await
}

#[when(expr = "{} turns message archiving {toggle}")]
async fn when_set_message_archiving(world: &mut TestWorld, name: String, state: ToggleState) {
    let token = world
        .members
        .get(&name)
        .expect("User must be created first")
        .1
        .clone();
    let res = set_message_archiving(&world.client, token, state.into()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the message archive retention to {duration}")]
async fn when_set_message_archive_retention(
    world: &mut TestWorld,
    name: String,
    duration: Duration,
) {
    let token = world
        .members
        .get(&name)
        .expect("User must be created first")
        .1
        .clone();
    let res = set_message_archive_retention(&world.client, token, duration.into()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} resets the Messaging configuration to its default value")]
async fn when_reset_messaging_confiuration(world: &mut TestWorld, name: String) {
    let token = world
        .members
        .get(&name)
        .expect("User must be created first")
        .1
        .clone();
    let res = reset_reset_messaging_confiuration(&world.client, token).await;
    world.result = Some(res.into());
}

#[when(expr = "{} resets the message archive retention to its default value")]
async fn when_reset_message_archive_retention(world: &mut TestWorld, name: String) {
    let token = world
        .members
        .get(&name)
        .expect("User must be created first")
        .1
        .clone();
    let res = reset_message_archive_retention(&world.client, token).await;
    world.result = Some(res.into());
}

#[then(expr = "message archiving is {toggle}")]
async fn then_message_archiving(world: &mut TestWorld, enabled: ToggleState) -> Result<(), Error> {
    let defaults = &world.config.server.defaults;
    let enabled = enabled.as_bool();

    // Check in database
    let db = world.db();
    let server_config = ServerConfigRepository::get(db)
        .await?
        .expect("Workspace not initialized");
    assert_eq!(server_config.message_archive_enabled(defaults), enabled);

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

#[then(expr = "the message archive retention is set to {duration}")]
async fn then_message_archive_retention(
    world: &mut TestWorld,
    duration: Duration,
) -> Result<(), DbErr> {
    let defaults = &world.config.server.defaults;
    let duration = duration.into();

    // Check in database
    let db = world.db();
    let server_config = ServerConfigRepository::get(db)
        .await?
        .expect("Workspace not initialized");
    assert_eq!(server_config.message_archive_retention(defaults), duration);

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

async fn set_file_uploading<'a>(
    client: &'a Client,
    token: SecretString,
    state: bool,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/config/allow-file-upload")
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .body(
            json!(SetFileUploadingRequest {
                file_upload_allowed: state,
            })
            .to_string(),
        )
        .dispatch()
        .await
}

async fn set_file_retention<'a>(
    client: &'a Client,
    token: SecretString,
    duration: Duration,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/config/file-retention")
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", token.expose_secret()),
        ))
        .body(
            json!(SetFileRetentionRequest {
                file_retention: duration.into(),
            })
            .to_string(),
        )
        .dispatch()
        .await
}

#[given(expr = "file uploading is {toggle}")]
async fn given_file_uploading(world: &mut TestWorld, state: ToggleState) -> Result<(), DbErr> {
    let db = world.db();
    let server_config = ServerConfigRepository::get(db)
        .await?
        .expect("Workspace should be initialized first");
    let mut model = server_config.into_active_model();
    model.file_upload_allowed = Set(Some(state.into()));
    model.update(db).await?;
    Ok(())
}

#[given(expr = "the file retention is set to {duration}")]
async fn given_file_retention(world: &mut TestWorld, duration: Duration) -> Result<(), DbErr> {
    let db = world.db();
    let server_config = ServerConfigRepository::get(db)
        .await?
        .expect("Workspace should be initialized first");
    let mut model = server_config.into_active_model();
    model.file_storage_retention = Set(Some(duration.into()));
    model.update(db).await?;
    Ok(())
}

#[when(expr = "{} turns file uploading {toggle}")]
async fn when_set_file_uploading(world: &mut TestWorld, name: String, state: ToggleState) {
    let token = world
        .members
        .get(&name)
        .expect("User must be created first")
        .1
        .clone();
    let res = set_file_uploading(&world.client, token, state.into()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the file retention to {duration}")]
async fn when_set_file_retention(world: &mut TestWorld, name: String, duration: Duration) {
    let token = world
        .members
        .get(&name)
        .expect("User must be created first")
        .1
        .clone();
    let res = set_file_retention(&world.client, token, duration.into()).await;
    world.result = Some(res.into());
}

#[then(expr = "file uploading is {toggle}")]
async fn then_file_uploading(world: &mut TestWorld, state: ToggleState) -> Result<(), DbErr> {
    let db = world.db();
    let defaults = &world.config.server.defaults;
    let server_config = ServerConfigRepository::get(db)
        .await?
        .expect("Workspace not initialized");

    assert_eq!(server_config.file_upload_allowed(defaults), state.as_bool());

    Ok(())
}

#[then(expr = "the file retention is set to {duration}")]
async fn then_file_retention(world: &mut TestWorld, duration: Duration) -> Result<(), DbErr> {
    let db = world.db();
    let defaults = &world.config.server.defaults;
    let server_config = ServerConfigRepository::get(db)
        .await?
        .expect("Workspace not initialized");

    assert_eq!(
        server_config.file_storage_retention(defaults),
        duration.into()
    );

    Ok(())
}
