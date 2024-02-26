// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod test_server_ctl;
mod v1;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cucumber::{then, World};
use entity::member;
use log::debug;
use prose_pod_api::pool::Db;
use prose_pod_api::server_ctl::ServerCtl;
use rocket::figment::Figment;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use serde::Deserialize;
use service::sea_orm::DatabaseConnection;
use test_server_ctl::{TestServerCtl, TestServerCtlState};
use tokio::runtime::Handle;
use tokio::task;

#[tokio::main]
async fn main() {
    // Run tests and ignore undefined steps
    TestWorld::run("tests/features").await;

    // Run and fail on undefined steps
    // TestWorld::cucumber()
    //     .fail_on_skipped()
    //     .run_and_exit("tests/features").await;
}

fn test_rocket(server_ctl: TestServerCtl) -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::figment())
        .merge(("databases.data.url", "sqlite::memory:"))
        .merge(("log_level", "off"))
        .merge(("databases.data.sqlx_logging", false))
        .merge(("databases.data.sql_log_level", "off"));
    prose_pod_api::custom_rocket(rocket::custom(figment))
        .manage(ServerCtl::new(Arc::new(Mutex::new(server_ctl))))
}

pub async fn rocket_test_client(server_ctl: TestServerCtl) -> Client {
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
                Self {
                    status: value.status(),
                    content_type: value.content_type(),
                    body: value.into_string().await,
                }
            })
        })
    }
}

#[derive(Debug, World)]
#[world(init = Self::new)]
struct TestWorld {
    server_state: Arc<Mutex<TestServerCtlState>>,
    client: Client,
    result: Option<Response>,
    /// Map a name to a member and an authorization token.
    members: HashMap<String, (member::Model, String)>,
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
}

impl TestWorld {
    async fn new() -> Self {
        let state = Arc::new(Mutex::new(TestServerCtlState::default()));

        Self {
            server_state: state.clone(),
            client: rocket_test_client(TestServerCtl::new(state)).await,
            result: None,
            members: HashMap::new(),
        }
    }
}

#[then("the call should succeed")]
fn then_response_ok(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.status, Status::Ok);
}

#[then("the response content type should be JSON")]
fn then_response_json(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.content_type, Some(ContentType::JSON));
}
