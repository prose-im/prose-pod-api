// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod workspace;

use crate::TestWorld;
use cucumber::{given, then, when};
use prose_pod_api::v1::InitRequest;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use serde_json::json;

pub const DEFAULT_WORKSPACE_NAME: &'static str = "Prose";

#[given("workspace has not been initialized")]
fn given_workspace_not_initialized(_world: &mut TestWorld) {
    // Do nothing, as a new test client is always empty
}

#[given("workspace has been initialized")]
async fn given_workspace_initialized(world: &mut TestWorld) {
    let res = init_workspace(&world.client, DEFAULT_WORKSPACE_NAME).await;
    assert_eq!(res.status(), Status::Ok);
}

#[then("the user should receive 'Prose Pod not initialized'")]
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
                "reason": "pod_not_initialized",
            })
            .to_string()
        )
    );
}

#[then("the user should receive 'Prose Pod already initialized'")]
async fn then_error_workspace_already_initialized(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.status, Status::Conflict);
    assert_eq!(res.content_type, Some(ContentType::JSON));
    assert_eq!(
        res.body,
        Some(
            json!({
                "reason": "pod_already_initialized",
            })
            .to_string()
        )
    );
}

// INIT

async fn init_workspace<'a>(
    client: &'a Client,
    name: &str,
) -> LocalResponse<'a> {
    client
        .post("/v1/init")
        .header(ContentType::JSON)
        .body(
            json!(InitRequest {
                workspace_name: name.to_string(),
            })
            .to_string(),
        )
        .dispatch()
        .await
}

#[when(expr = "a user ititializes a workspace named {string}")]
async fn when_workspace_init(
    world: &mut TestWorld,
    name: String,
) {
    let res = init_workspace(&world.client, &name).await;
    world.result = Some(res.into());
}

#[tokio::test]
async fn test_init_settings() -> Result<(), Box<dyn Error>> {
    let client = rocket_test_client().await;

    let workspace_name = DEFAULT_WORKSPACE_NAME;
    let body = serde_json::to_string(&InitRequest {
        workspace_name: workspace_name.to_string(),
    })?;

    let response = client.post("/v1/init").body(&body).dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = response.into_string().await.expect("Response Body");
    let res: InitResponse = serde_json::from_str(&body.as_str()).expect("Valid Response");

    assert_eq!(res.workspace_name, workspace_name);

    Ok(())
}
