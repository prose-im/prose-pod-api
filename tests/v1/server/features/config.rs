// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::{given, then, when};
use migration::DbErr;
use prose_pod_api::v1::server::features::config::*;
use rocket::http::{ContentType, Header};
use rocket::local::asynchronous::{Client, LocalResponse};
use serde_json::json;
use service::sea_orm::{ActiveModelTrait as _, IntoActiveModel, Set};
use service::Query;

use crate::v1::{Duration, ToggleState};
use crate::TestWorld;

// MESSAGE ARCHIVING

async fn set_message_archiving<'a>(
    client: &'a Client,
    token: String,
    state: bool,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/features/config/store-message-archive")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
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
    token: String,
    duration: Duration,
) -> LocalResponse<'a> {
    client
        .put("/v1/server/features/config/message-archive-retention")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", format!("Bearer {token}")))
        .body(
            json!(SetMessageArchiveRetentionRequest {
                message_archive_retention: duration.into(),
            })
            .to_string(),
        )
        .dispatch()
        .await
}

#[given(expr = "message archiving is {toggle}")]
async fn given_message_archiving(world: &mut TestWorld, state: ToggleState) -> Result<(), DbErr> {
    let db = world.db();
    let server_config = Query::server_config(db)
        .await?
        .expect("Workspace should be initialized first");
    let mut model = server_config.into_active_model();
    model.message_archive_enabled = Set(state.into());
    model.update(db).await?;
    Ok(())
}

#[given(expr = "the message archive retention is set to {duration}")]
async fn given_message_archive_retention(
    world: &mut TestWorld,
    duration: Duration,
) -> Result<(), DbErr> {
    let db = world.db();
    let server_config = Query::server_config(db)
        .await?
        .expect("Workspace should be initialized first");
    let mut model = server_config.into_active_model();
    model.message_archive_retention = Set(duration.into());
    model.update(db).await?;
    Ok(())
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

#[then(expr = "message archiving is {toggle}")]
async fn then_message_archiving(world: &mut TestWorld, state: ToggleState) -> Result<(), DbErr> {
    let db = world.db();
    let server_config = Query::server_config(db)
        .await?
        .expect("Workspace not initialized");
    assert_eq!(server_config.message_archive_enabled, state.as_bool());
    Ok(())
}

#[then(expr = "the message archive retention is set to {duration}")]
async fn then_message_archive_retention(
    world: &mut TestWorld,
    duration: Duration,
) -> Result<(), DbErr> {
    let db = world.db();
    let server_config = Query::server_config(db)
        .await?
        .expect("Workspace not initialized");
    assert_eq!(server_config.message_archive_retention, duration.into());
    Ok(())
}
