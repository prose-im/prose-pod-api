// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_resolver::Name as DomainName;
use prose_pod_api::features::{
    init::*, server_config::dtos::InitServerConfigRequest, workspace_details::InitWorkspaceRequest,
};
use service::{
    models::Url,
    pod_config::{
        NetworkAddressCreateForm, PodConfigCreateForm, PodConfigRepository, PodConfigUpdateForm,
    },
    server_config::server_config_controller,
    workspace::{workspace_controller, Workspace},
};

use super::prelude::*;

pub const DEFAULT_WORKSPACE_NAME: &'static str = "Prose";

#[given("the Prose Pod has not been initialized")]
fn given_pod_not_initialized(world: &mut TestWorld) {
    given_workspace_not_initialized(world);
    given_server_config_not_initialized(world);
}

#[given("the Prose Pod has been initialized")]
async fn given_pod_initialized(world: &mut TestWorld) -> Result<(), Error> {
    given_server_config_initialized(world).await?;
    given_workspace_initialized(world).await?;
    given_pod_config_initialized(world).await?;
    Ok(())
}

#[given(expr = "the Prose Pod has been initialized for {domain_name}")]
async fn given_pod_initialized_for_domain(
    world: &mut TestWorld,
    domain: parameters::DomainName,
) -> Result<(), Error> {
    given_server_domain(world, domain).await?;
    given_pod_initialized(world).await?;
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

#[given("the server config has not been initialized")]
fn given_server_config_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("the server config has been initialized")]
async fn given_server_config_initialized(world: &mut TestWorld) -> Result<(), Error> {
    let form = ServerConfigCreateForm {
        domain: JidDomain::from_str(&world.initial_server_domain).unwrap(),
    };

    server_config_controller::init_server_config(
        world.db(),
        &world.server_ctl,
        &world.app_config(),
        &world.auth_service,
        &world.secrets_store,
        form,
    )
    .await?;

    world.reset_server_ctl_counts();
    Ok(())
}

#[given("the Pod config has been initialized")]
async fn given_pod_config_initialized(world: &mut TestWorld) -> Result<(), Error> {
    let base_domain: DomainName = world
        .initial_server_domain
        .parse()
        .expect("Invalid server domain");
    PodConfigRepository::create(
        world.db(),
        PodConfigCreateForm {
            dashboard_url: Some(
                Url::parse(&format!("https://admin.{base_domain}")).expect("Invalid dashboard URL"),
            ),
            address: NetworkAddressCreateForm {
                hostname: Some(base_domain.to_string()),
                ..Default::default()
            },
        },
    )
    .await?;

    Ok(())
}

#[given(expr = "the XMPP server domain is {domain_name}")]
async fn given_server_domain(
    world: &mut TestWorld,
    domain: parameters::DomainName,
) -> Result<(), Error> {
    let domain = JidDomain::from_str(&domain.to_string()).expect("Invalid domain");
    if let Some(server_manager) = world.server_manager().await? {
        server_manager.set_domain(&domain).await?;
    }
    world.initial_server_domain = domain;

    Ok(())
}

#[given(expr = "the dashboard URL is {domain_name}")]
async fn given_dashboard_url(
    world: &mut TestWorld,
    domain: parameters::DomainName,
) -> Result<(), Error> {
    PodConfigRepository::set(
        world.db(),
        PodConfigUpdateForm {
            dashboard_url: Some(Some(
                Url::parse(&domain.to_string()).expect("Invalid dashboard URL"),
            )),
            ..Default::default()
        },
    )
    .await?;
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

async fn init_server_config(api: &TestServer, domain: &str) -> TestResponse {
    let domain = JidDomain::from_str(&domain).expect("Invalid domain");
    api.put("/v1/server/config")
        .json(&json!(InitServerConfigRequest { domain }))
        .await
}

async fn init_first_account(api: &TestServer, node: &JidNode, nickname: &String) -> TestResponse {
    api.put("/v1/init/first-account")
        .add_header(CONTENT_TYPE, "application/json")
        .json(&json!(InitFirstAccountRequest {
            username: node.to_owned(),
            password: SecretString::from("test.password").into(),
            nickname: nickname.to_owned(),
        }))
        .await
}

#[when(expr = "someone initializes a workspace named {string}")]
async fn when_init_workspace(world: &mut TestWorld, name: String) {
    let res = init_workspace(world.api(), &name).await;
    world.result = Some(res);
}

#[when(expr = "someone initializes the server at <{}>")]
async fn when_init_server_config(world: &mut TestWorld, domain: String) {
    let res = init_server_config(world.api(), &domain).await;
    world.result = Some(res);
}

#[when(expr = "someone creates the first account {string} with node {string}")]
async fn when_init_first_account(world: &mut TestWorld, nickname: String, node: JidNode) {
    let res = init_first_account(world.api(), &node, &nickname).await;
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

#[then("the user should receive 'Server config not initialized'")]
async fn then_error_server_config_not_initialized(world: &mut TestWorld) {
    let res = world.result();
    res.assert_status(StatusCode::PRECONDITION_FAILED);
    assert_eq!(
        res.header(CONTENT_TYPE),
        "application/json",
        "Content type (body: {:#?})",
        res.text()
    );
    res.assert_json(&json!({
        "error": "server_config_not_initialized",
        "message": "XMPP server not initialized.",
        "recovery_suggestions": ["Call `PUT /v1/server/config` to initialize it."],
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

#[then("the user should receive 'Server config already initialized'")]
async fn then_error_server_config_already_initialized(world: &mut TestWorld) {
    let res = world.result();
    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.header(CONTENT_TYPE), "application/json");
    res.assert_json(&json!({
        "error": "server_config_already_initialized",
        "message": "XMPP server already initialized.",
    }));
}
