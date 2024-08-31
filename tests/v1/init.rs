// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use cucumber::{given, then, when};
use prose_pod_api::error::Error;
use prose_pod_api::v1::init::{
    InitFirstAccountRequest, InitServerConfigRequest, InitWorkspaceRequest,
};
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use secrecy::SecretString;
use serde_json::json;
use service::controllers::init_controller::WorkspaceCreateForm;
use service::model::{JidDomain, JidNode};
use service::repositories::ServerConfigCreateForm;

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
    given_server_config_initialized(world).await?;
    given_workspace_initialized(world).await?;
    Ok(())
}

#[given("the workspace has not been initialized")]
fn given_workspace_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("the workspace has been initialized")]
async fn given_workspace_initialized(world: &mut TestWorld) -> Result<(), Error> {
    let form = WorkspaceCreateForm {
        name: DEFAULT_WORKSPACE_NAME.to_string(),
        accent_color: None,
    };

    world
        .init_controller()
        .init_workspace(
            &world.app_config,
            &world.secrets_store,
            &world.xmpp_service,
            &world.server_config().await?,
            form,
        )
        .await?;

    Ok(())
}

#[given("the server config has not been initialized")]
fn given_server_config_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("the server config has been initialized")]
async fn given_server_config_initialized(world: &mut TestWorld) -> Result<(), Error> {
    let form = ServerConfigCreateForm {
        domain: JidDomain::from_str(DEFAULT_DOMAIN).unwrap(),
    };

    world
        .init_controller()
        .init_server_config(
            &world.server_ctl,
            &world.app_config,
            &world.auth_service,
            &world.secrets_store,
            form,
        )
        .await?;

    world.reset_server_ctl_counts();
    Ok(())
}

#[given(expr = "the XMPP server domain is {}")]
async fn given_server_domain(world: &mut TestWorld, domain: String) -> Result<(), Error> {
    let server_manager = world.server_manager().await?;
    let domain = JidDomain::from_str(&domain).expect("Invalid domain");
    server_manager.set_domain(&domain).await?;
    Ok(())
}

#[given("nothing has changed since the initialization of the workspace")]
fn given_nothing_changed(_world: &mut TestWorld) {
    // Do nothing, even though we could performs checks
}

#[given("the Prose Pod address has not been initialized")]
fn given_pod_address_not_initialized(_world: &mut TestWorld) {
    // Do nothing, even though we could performs checks
}

async fn init_workspace<'a>(client: &'a Client, name: &str) -> LocalResponse<'a> {
    client
        .put("/v1/workspace")
        .json(&json!(InitWorkspaceRequest {
            name: name.to_owned(),
            accent_color: None,
        }))
        .dispatch()
        .await
}

async fn init_server_config<'a>(client: &'a Client, domain: &str) -> LocalResponse<'a> {
    let domain = JidDomain::from_str(&domain).expect("Invalid domain");
    client
        .put("/v1/server/config")
        .json(&json!(InitServerConfigRequest { domain }))
        .dispatch()
        .await
}

async fn init_first_account<'a>(
    client: &'a Client,
    node: &JidNode,
    nickname: &String,
) -> LocalResponse<'a> {
    client
        .put("/v1/init/first-account")
        .header(ContentType::JSON)
        .body(
            json!(InitFirstAccountRequest {
                username: node.to_owned(),
                password: SecretString::new("test.password".to_string()).into(),
                nickname: nickname.to_owned(),
            })
            .to_string(),
        )
        .dispatch()
        .await
}

#[when(expr = "someone initializes a workspace named {string}")]
async fn when_init_workspace(world: &mut TestWorld, name: String) {
    let res = init_workspace(world.client(), &name).await;
    world.result = Some(res.into());
}

#[when(expr = "someone initializes the server at <{text}>")]
async fn when_init_server_config(world: &mut TestWorld, domain: Text) {
    let res = init_server_config(world.client(), &domain).await;
    world.result = Some(res.into());
}

#[when(expr = "someone creates the first account {string} with node {string}")]
async fn when_init_first_account(world: &mut TestWorld, nickname: String, node: JidNode) {
    let res = init_first_account(world.client(), &node, &nickname).await;
    world.result = Some(res.into());
}

#[then(expr = "the error reason should be {string}")]
async fn then_error_reason(world: &mut TestWorld, reason: String) -> Result<(), serde_json::Error> {
    let res = world.result();
    assert_eq!(res.content_type, Some(ContentType::JSON));
    let body = serde_json::Value::from_str(res.body.as_ref().expect("No body found."))?;
    assert_eq!(body["reason"].as_str(), Some(reason.as_str()));
    Ok(())
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

#[then("the user should receive 'First account already created'")]
async fn then_error_first_account_already_created(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.status, Status::Conflict);
    assert_eq!(res.content_type, Some(ContentType::JSON));
    assert_eq!(
        res.body,
        Some(
            json!({
                "reason": "first_account_already_created",
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
