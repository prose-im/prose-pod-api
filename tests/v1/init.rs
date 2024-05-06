// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::{given, then, when};
use entity::model::JIDNode;
use entity::server_config;
use prose_pod_api::error::Error;
use prose_pod_api::guards::UnauthenticatedServerManager;
use prose_pod_api::v1::init::{
    InitFirstAccountRequest, InitServerConfigRequest, InitWorkspaceRequest,
};
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use serde_json::json;
use service::sea_orm::Set;
use service::{Mutation, ServerCtl};

use crate::cucumber_parameters::Text;
use crate::TestWorld;

pub const DEFAULT_WORKSPACE_NAME: &'static str = "Prose";
pub const DEFAULT_DOMAIN: &'static str = "prose.test.org";

#[given("the Prose Pod has not been initialized")]
fn given_pod_not_initialized(world: &mut TestWorld) {
    given_workspace_not_initialized(world);
    given_server_config_not_initialized(world);
}

#[given("the Prose Pod has been initialized")]
async fn given_pod_initialized(world: &mut TestWorld) -> Result<(), Error> {
    given_workspace_initialized(world).await?;
    given_server_config_initialized(world).await?;
    Ok(())
}

#[given("the workspace has not been initialized")]
fn given_workspace_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("the workspace has been initialized")]
async fn given_workspace_initialized(world: &mut TestWorld) -> Result<(), Error> {
    let db = world.db();
    let form = entity::workspace::ActiveModel {
        name: Set(DEFAULT_WORKSPACE_NAME.to_string()),
        ..Default::default()
    };
    Mutation::create_workspace(db, form).await?;
    Ok(())
}

#[given("the server config has not been initialized")]
fn given_server_config_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("the server config has been initialized")]
async fn given_server_config_initialized(world: &mut TestWorld) -> Result<(), Error> {
    let db = world.db();
    let form = server_config::ActiveModel {
        domain: Set(DEFAULT_DOMAIN.to_string()),
        ..Default::default()
    };
    UnauthenticatedServerManager::init_server_config(
        db,
        &ServerCtl::new(world.server_ctl.clone()),
        world.config.as_ref(),
        form,
    )
    .await?;
    Ok(())
}

#[given(expr = "the XMPP server domain is <{text}>")]
async fn given_server_domain(world: &mut TestWorld, domain: Text) -> Result<(), Error> {
    let server_manager = world.server_manager().await?;
    server_manager.set_domain(&domain).await?;
    Ok(())
}

#[given("nothing has changed since the initialization of the workspace")]
fn given_nothing_changed(_world: &mut TestWorld) {
    // Do nothing, even though we could performs checks
}

async fn init_workspace<'a>(client: &'a Client, name: &str) -> LocalResponse<'a> {
    client
        .put("/v1/workspace")
        .header(ContentType::JSON)
        .body(
            json!(InitWorkspaceRequest {
                name: name.to_owned(),
                ..Default::default()
            })
            .to_string(),
        )
        .dispatch()
        .await
}

async fn init_server_config<'a>(client: &'a Client, domain: &str) -> LocalResponse<'a> {
    client
        .put("/v1/server/config")
        .header(ContentType::JSON)
        .body(
            json!(InitServerConfigRequest {
                domain: domain.to_owned()
            })
            .to_string(),
        )
        .dispatch()
        .await
}

async fn init_first_member<'a>(
    client: &'a Client,
    node: &JIDNode,
    nickname: &String,
) -> LocalResponse<'a> {
    client
        .put("/v1/init/first-member")
        .header(ContentType::JSON)
        .body(
            json!(InitFirstAccountRequest {
                username: node.to_owned(),
                password: "test.password".to_owned(),
                nickname: nickname.to_owned(),
            })
            .to_string(),
        )
        .dispatch()
        .await
}

#[when(expr = "someone initializes a workspace named {string}")]
async fn when_init_workspace(world: &mut TestWorld, name: String) {
    let res = init_workspace(&world.client, &name).await;
    world.result = Some(res.into());
}

#[when(expr = "someone initializes the server at <{text}>")]
async fn when_init_server_config(world: &mut TestWorld, domain: Text) {
    let res = init_server_config(&world.client, &domain).await;
    world.result = Some(res.into());
}

#[when(expr = "someone creates the first member {string} with node {string}")]
async fn when_init_first_member(world: &mut TestWorld, nickname: String, node: JIDNode) {
    let res = init_first_member(&world.client, &node, &nickname).await;
    world.result = Some(res.into());
}

#[then("the user should receive 'Workspace not initialized'")]
async fn then_error_workspace_not_initialized(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.status, Status::BadRequest, "Status");
    assert_eq!(
        res.content_type,
        Some(ContentType::JSON),
        "Content type (body: {:#?})",
        res.body
    );
    assert_eq!(
        res.body,
        Some(
            json!({
                "reason": "workspace_not_initialized",
            })
            .to_string()
        )
    );
}

#[then("the user should receive 'Workspace already initialized'")]
async fn then_error_workspace_already_initialized(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.status, Status::Conflict);
    assert_eq!(res.content_type, Some(ContentType::JSON));
    assert_eq!(
        res.body,
        Some(
            json!({
                "reason": "workspace_already_initialized",
            })
            .to_string()
        )
    );
}

#[then("the user should receive 'Server config not initialized'")]
async fn then_error_server_config_not_initialized(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.status, Status::BadRequest, "Status");
    assert_eq!(
        res.content_type,
        Some(ContentType::JSON),
        "Content type (body: {:#?})",
        res.body
    );
    assert_eq!(
        res.body,
        Some(
            json!({
                "reason": "server_config_not_initialized",
            })
            .to_string()
        )
    );
}

#[then("the user should receive 'Server config already initialized'")]
async fn then_error_server_config_already_initialized(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.status, Status::Conflict);
    assert_eq!(res.content_type, Some(ContentType::JSON));
    assert_eq!(
        res.body,
        Some(
            json!({
                "reason": "server_config_already_initialized",
            })
            .to_string()
        )
    );
}
