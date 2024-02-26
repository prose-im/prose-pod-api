// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod server;
pub mod workspace;

use std::str::FromStr;

use crate::TestWorld;
use cucumber::codegen::Regex;
use cucumber::{given, then, when, Parameter};
use migration::sea_orm::{ActiveModelTrait, Set};
use migration::DbErr;
use prose_pod_api::v1::InitRequest;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use serde_json::json;

pub const DEFAULT_WORKSPACE_NAME: &'static str = "Prose";

#[given(regex = r"^(.+) is an admin$")]
async fn given_admin(world: &mut TestWorld, name: String) -> Result<(), DbErr> {
    let db = world.db();
    let model = ::entity::member::ActiveModel {
        id: Set(format!("{}@test.org", name.to_lowercase())),
        ..Default::default()
    };
    let model = model.insert(db).await?;

    let response = login(&world.client).await;
    let token = response.into_string().await.expect("Could not log in");

    world.members.insert(name, (model, token));

    Ok(())
}

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

// Custom Cucumber parameters
// See <https://cucumber-rs.github.io/cucumber/current/writing/capturing.html#custom-parameters>

#[derive(Debug, Parameter)]
#[param(name = "toggle", regex = "on|off|enabled|disabled")]
enum ToggleState {
    Enabled,
    Disabled,
}

impl ToggleState {
    fn as_bool(&self) -> bool {
        match self {
            Self::Enabled => true,
            Self::Disabled => false,
        }
    }
}

impl Into<bool> for ToggleState {
    fn into(self) -> bool {
        self.as_bool()
    }
}

impl FromStr for ToggleState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "on" | "enabled" => Self::Enabled,
            "off" | "disabled" => Self::Disabled,
            invalid => return Err(format!("Invalid `ToggleState`: {invalid}")),
        })
    }
}

#[derive(Debug, Parameter)]
#[param(
    name = "duration",
    regex = r"\d+ (?:year|month|week|day)s?(?: \d+ (?:year|month|week|day)s?)*"
)]
struct Duration(String);

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let patterns = vec![
            (r"(\d+) years?", 'Y'),
            (r"(\d+) months?", 'M'),
            (r"(\d+) weeks?", 'W'),
            (r"(\d+) days?", 'D'),
        ];

        let mut value = "P".to_string();
        for (pattern, designator) in patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(s) {
                value.push_str(captures.get(1).unwrap().as_str());
                value.push(designator);
            }
        }

        match value.as_str() {
            "P" => Err(format!("Invalid `Duration`: {s}")),
            _ => Ok(Self(value)),
        }
    }
}

impl ToString for Duration {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

// LOGIN

async fn login<'a>(client: &'a Client) -> LocalResponse<'a> {
    client
        .post("/v1/login")
        .header(ContentType::JSON)
        .dispatch()
        .await
}

// INIT

async fn init_workspace<'a>(client: &'a Client, name: &str) -> LocalResponse<'a> {
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
async fn when_workspace_init(world: &mut TestWorld, name: String) {
    let res = init_workspace(&world.client, &name).await;
    world.result = Some(res.into());
}

#[tokio::test]
async fn test_init_workspace() -> Result<(), Box<dyn Error>> {
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
