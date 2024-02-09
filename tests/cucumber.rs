// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod v1;

use cucumber::{then, World};
use log::debug;
use rocket::figment::Figment;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use rocket::{Build, Rocket};
use serde::Deserialize;
use tokio::runtime::Handle;
use tokio::task;

#[tokio::main]
async fn main() {
    TestWorld::run("tests/features").await;
}

fn test_rocket() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::figment())
        .merge(("databases.data.url", "sqlite::memory:"))
        .merge(("log_level", "off"))
        .merge(("databases.data.sqlx_logging", false))
        .merge(("databases.data.sql_log_level", "off"));
    prose_pod_api::custom_rocket(rocket::custom(figment))
}

pub async fn rocket_test_client() -> Client {
    debug!("Creating Rocket test client...");
    Client::tracked(test_rocket())
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
    client: Client,
    result: Option<Response>,
}

impl TestWorld {
    fn result(&mut self) -> &mut Response {
        match &mut self.result {
            Some(res) => res,
            None => panic!("A call must be made before"),
        }
    }
}

impl TestWorld {
    async fn new() -> Self {
        Self {
            client: rocket_test_client().await,
            result: None,
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
