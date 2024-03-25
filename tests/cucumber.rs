// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod cucumber_parameters;
mod dummy_server_ctl;
mod v1;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cucumber::{then, World};
use cucumber_parameters::HTTPStatus;
use dummy_server_ctl::{DummyServerCtl, DummyServerCtlState};
use entity::model::EmailAddress;
use entity::{member, member_invite};
use log::debug;
use prose_pod_api::guards::{Db, JWTKey, JWTService};
use rocket::figment::Figment;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use serde::Deserialize;
use service::sea_orm::DatabaseConnection;
use service::ServerCtl;
use tokio::runtime::Handle;
use tokio::task;
// use tracing_subscriber::{
//     filter::{self, LevelFilter},
//     fmt::format::{self, Format},
//     layer::{Layer, SubscriberExt as _},
// };

#[tokio::main]
async fn main() {
    // Run tests and ignore undefined steps
    // TestWorld::run("tests/features").await;

    // Run tests and ignore undefined steps, but show logs
    // NOTE: Needs the "tracing" feature enabled for `cucumber`
    TestWorld::cucumber()
        .init_tracing()
        // .configure_and_init_tracing(
        //     format::DefaultFields::new(),
        //     Format::default(),
        //     |fmt_layer| {
        //         tracing_subscriber::registry()
        //             .with(
        //                 filter::Targets::new()
        //                     .with_targets(vec![
        //                         ("rocket", LevelFilter::WARN),
        //                         ("sea_orm_migration", LevelFilter::WARN),
        //                         ("rocket::server", LevelFilter::TRACE),
        //                     ])
        //                     .and_then(fmt_layer)
        //             )
        //     },
        // )
        .run("tests/features")
        .await;

    // Run and fail on undefined steps
    // TestWorld::cucumber()
    //     .fail_on_skipped()
    //     .run_and_exit("tests/features").await;
}

fn test_rocket(server_ctl: DummyServerCtl) -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::figment())
        .merge(("databases.data.url", "sqlite::memory:"))
        .merge(("log_level", "off"))
        .merge(("databases.data.sqlx_logging", false))
        .merge(("databases.data.sql_log_level", "off"));
    prose_pod_api::custom_rocket(rocket::custom(figment))
        .manage(JWTService::new(JWTKey::custom("test_key")))
        .manage(ServerCtl::new(Arc::new(Mutex::new(server_ctl))))
}

pub async fn rocket_test_client(server_ctl: DummyServerCtl) -> Client {
    debug!("Creating Rocket test client...");
    Client::tracked(test_rocket(server_ctl))
        .await
        .expect("valid rocket instance")
}

#[derive(Debug)]
struct Response {
    status: Status,
    content_type: Option<ContentType>,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl Response {
    fn body(&mut self) -> &String {
        match &self.body {
            Some(res) => res,
            None => panic!("Response had no body"),
        }
    }
    fn body_into<T>(&mut self) -> T
    where
        for<'de> T: Deserialize<'de>,
    {
        serde_json::from_str(&self.body().as_str()).expect("Valid Response")
    }
}

impl From<LocalResponse<'_>> for Response {
    fn from(value: LocalResponse) -> Self {
        task::block_in_place(|| {
            Handle::current().block_on(async move {
                let mut headers: HashMap<String, String> = HashMap::new();
                for header in value.headers().iter() {
                    headers.insert(header.name().to_string(), header.value().to_string());
                }
                Self {
                    status: value.status(),
                    content_type: value.content_type(),
                    body: value.into_string().await,
                    headers,
                }
            })
        })
    }
}

#[derive(Debug, World)]
#[world(init = Self::new)]
struct TestWorld {
    server_state: Arc<Mutex<DummyServerCtlState>>,
    client: Client,
    result: Option<Response>,
    /// Map a name to a member and an authorization token.
    members: HashMap<String, (member::Model, String)>,
    /// Map an email address to an invite.
    member_invites: HashMap<EmailAddress, member_invite::Model>,
    scenario_invite: Option<(EmailAddress, member_invite::Model)>,
}

impl TestWorld {
    fn result(&mut self) -> &mut Response {
        match &mut self.result {
            Some(res) => res,
            None => panic!("A call must be made before"),
        }
    }

    fn db(&self) -> &DatabaseConnection {
        &Db::fetch(&self.client.rocket()).unwrap().conn
    }

    fn token(&self, user: String) -> String {
        self.members
            .get(&user)
            .expect("User must be created first")
            .1
            .clone()
    }

    fn scenario_invite(&self) -> (EmailAddress, member_invite::Model) {
        self.scenario_invite
            .as_ref()
            .expect("Invite must be created first")
            .clone()
    }

    fn invite(&self, email_address: EmailAddress) -> member_invite::Model {
        self.member_invites
            .get(&email_address)
            .expect("Invite must be created first")
            .clone()
    }
}

impl TestWorld {
    async fn new() -> Self {
        let state = Arc::new(Mutex::new(DummyServerCtlState::default()));

        Self {
            server_state: state.clone(),
            client: rocket_test_client(DummyServerCtl::new(state)).await,
            result: None,
            members: HashMap::new(),
            member_invites: HashMap::new(),
            scenario_invite: None,
        }
    }
}

#[then("the call should succeed")]
fn then_response_ok(world: &mut TestWorld) {
    let res = world.result();
    assert!(
        (200..300).contains(&res.status.code),
        "Status is not a success ({:#?})",
        res
    );
}

#[then("the call should not succeed")]
fn then_response_not_ok(world: &mut TestWorld) {
    let res = world.result();
    assert!(
        !(200..300).contains(&res.status.code),
        "Status is not a failure ({:#?})",
        res
    );
}

#[then("the response content type should be JSON")]
fn then_response_json(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.content_type, Some(ContentType::JSON));
}

#[then(expr = "the HTTP status code should be {status}")]
fn then_response_http_status(world: &mut TestWorld, status: HTTPStatus) {
    let res = world.result();
    assert_eq!(res.status, *status);
}

#[then(expr = "the response should contain a {string} HTTP header")]
fn then_response_headers_contain(world: &mut TestWorld, header_name: String) {
    let res = world.result();
    assert!(
        res.headers.contains_key(&header_name),
        "No '{}' header found. Headers: {:?}",
        &header_name,
        &res.headers.keys()
    );
}

#[then(expr = "the {string} header should contain {string}")]
fn then_response_header_equals(world: &mut TestWorld, header_name: String, header_value: String) {
    let res = world.result();
    assert_eq!(
        res.headers.get(&header_name),
        Some(&header_value),
        "No '{}' header found. Headers: {:?}",
        &header_name,
        &res.headers.keys()
    );
}
