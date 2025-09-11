// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use prose_pod_api::features::{
    auth::models::Password, init::*, workspace_details::InitWorkspaceRequest,
};
use service::{
    members::Nickname,
    workspace::{workspace_controller, Workspace},
    xmpp::server_manager,
};

use super::prelude::*;

pub const DEFAULT_WORKSPACE_NAME: &'static str = "Prose";

#[given("the Prose Pod has not been initialized")]
fn given_pod_not_initialized(world: &mut TestWorld) {
    given_workspace_not_initialized(world);
    given_xmpp_server_not_initialized(world);
}

#[given("the Prose Pod has been initialized")]
async fn given_pod_initialized(world: &mut TestWorld) -> Result<(), Error> {
    given_xmpp_server_initialized(world).await?;
    given_workspace_initialized(world).await?;
    Ok(())
}

#[given("the workspace has not been initialized")]
fn given_workspace_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("the workspace has been initialized")]
#[given("the Workspace has been initialized")]
async fn given_workspace_initialized(world: &mut TestWorld) -> Result<(), Error> {
    let workspace = Workspace {
        name: DEFAULT_WORKSPACE_NAME.to_string(),
        accent_color: None,
        icon: None,
    };

    workspace_controller::init_workspace(&world.workspace_service().await, workspace).await?;

    Ok(())
}

#[given("the XMPP server has not been initialized")]
fn given_xmpp_server_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("the XMPP server has been initialized")]
async fn given_xmpp_server_initialized(world: &mut TestWorld) -> anyhow::Result<()> {
    // Initialize XMPP server configuration.
    world.server_config_manager().reload().await?;

    // Register OAuth 2.0 client.
    (world.auth_service.register_oauth2_client().await)
        .context("Could not register OAuth 2.0 client")?;

    // Create service XMPP accounts.
    server_manager::create_service_accounts(
        &world.server_ctl,
        &world.app_config().clone(),
        &world.auth_service,
        &world.secrets_store(),
    )
    .await
    .context("Could not create service XMPP account")?;

    // Add the Workspace XMPP account to everyone’s rosters so they receive
    // Workspace icon updates.
    let workspace_jid = world.app_config().workspace_jid();
    (world.server_ctl.add_team_member(&workspace_jid).await)
        .context("Could not add the Workspace to the team")?;

    world.reset_server_ctl_counts();
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

async fn init_workspace(api: &TestServer, name: &str) -> TestResponse {
    api.put("/v1/workspace")
        .json(&json!(InitWorkspaceRequest {
            name: name.to_owned(),
            accent_color: None,
        }))
        .await
}

async fn init_first_account(api: &TestServer, node: &JidNode, nickname: String) -> TestResponse {
    api.put("/v1/init/first-account")
        .add_header(CONTENT_TYPE, "application/json")
        .json(&json!(InitFirstAccountRequest {
            username: node.to_owned(),
            password: Password::from("test.password"),
            nickname: Nickname::from_string_unsafe(nickname),
        }))
        .await
}

#[when(expr = "someone initializes a workspace named {string}")]
async fn when_init_workspace(world: &mut TestWorld, name: String) {
    let res = init_workspace(world.api(), &name).await;
    world.result = Some(res);
}

#[when(expr = "someone creates the first account {string} with node {string}")]
async fn when_init_first_account(world: &mut TestWorld, nickname: String, node: JidNode) {
    let res = init_first_account(world.api(), &node, nickname).await;
    world.result = Some(res);
}

#[then(expr = "the error code should be {string}")]
async fn then_error_reason(world: &mut TestWorld, reason: String) -> Result<(), serde_json::Error> {
    let res = world.result();
    assert_eq!(res.header(CONTENT_TYPE), "application/json");
    let body = serde_json::Value::from_str(&res.text())?;
    assert_eq!(body["error"].as_str(), Some(reason.as_str()));
    Ok(())
}

#[then("the user should receive 'Workspace not initialized: No vCard'")]
async fn then_error_workspace_not_initialized(world: &mut TestWorld) {
    let res = world.result();
    res.assert_status(StatusCode::PRECONDITION_FAILED);
    assert_eq!(
        res.header(CONTENT_TYPE),
        "application/json",
        "Content type (body: {:#?})",
        res.text()
    );
    res.assert_json(&json!({
        "error": "workspace_not_initialized",
        "message": "Workspace not initialized: No vCard.",
        "recovery_suggestions": [
            "Call `PUT /v1/workspace` to initialize it.",
        ]
    }));
}

#[then("the user should receive 'Workspace already initialized'")]
async fn then_error_workspace_already_initialized(world: &mut TestWorld) {
    let res = world.result();
    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.header(CONTENT_TYPE), "application/json");
    res.assert_json(&json!({
        "error": "workspace_already_initialized",
        "message": "Workspace already initialized.",
    }));
}

#[then("the user should receive 'First account already created'")]
async fn then_error_first_account_already_created(world: &mut TestWorld) {
    let res = world.result();
    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.header(CONTENT_TYPE), "application/json");
    res.assert_json(&json!({
        "error": "first_account_already_created",
        "message": "First account already created.",
    }));
}
