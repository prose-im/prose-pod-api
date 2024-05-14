// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod cucumber_parameters;
mod prelude;
mod v1;

use self::prelude::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use cucumber::{then, World};
use cucumber_parameters::HTTPStatus;
use dummy_notifier::DummyNotifier;
use dummy_server_ctl::DummyServerCtl;
use entity::model::EmailAddress;
use entity::{member, workspace_invitation};
use log::debug;
use prose_pod_api::error::Error;
use prose_pod_api::guards::{Db, JWTKey, JWTService, ServerManager, UnauthenticatedServerManager};
use rocket::figment::Figment;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use serde::Deserialize;
use service::config::Config;
use service::notifier::AnyNotifier;
use service::sea_orm::DatabaseConnection;
use service::ServerCtl;
use service::{dependencies, Query};
use tokio::runtime::Handle;
use tokio::task;
use uuid::Uuid;
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

fn test_rocket(
    config: &Config,
    server_ctl: Arc<Mutex<DummyServerCtl>>,
    notifier: Arc<Mutex<DummyNotifier>>,
) -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::figment())
        .merge(("databases.data.url", "sqlite::memory:"))
        .merge(("log_level", "off"))
        .merge(("databases.data.sqlx_logging", false))
        .merge(("databases.data.sql_log_level", "off"));
    prose_pod_api::custom_rocket(
        rocket::custom(figment),
        config.to_owned(),
        ServerCtl::new(server_ctl),
    )
    .manage(JWTService::new(JWTKey::custom("test_key")))
    .manage(dependencies::Notifier::from(AnyNotifier::new(notifier)))
}

pub async fn rocket_test_client(
    config: Arc<Config>,
    server_ctl: Arc<Mutex<DummyServerCtl>>,
    notifier: Arc<Mutex<DummyNotifier>>,
) -> Client {
    debug!("Creating Rocket test client...");
    Client::tracked(test_rocket(config.as_ref(), server_ctl, notifier))
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
pub struct TestWorld {
    config: Arc<Config>,
    server_ctl: Arc<Mutex<DummyServerCtl>>,
    notifier: Arc<Mutex<DummyNotifier>>,
    client: Client,
    result: Option<Response>,
    /// Map a name to a member and an authorization token.
    members: HashMap<String, (member::Model, String)>,
    /// Map an email address to an invitation.
    workspace_invitations: HashMap<EmailAddress, workspace_invitation::Model>,
    scenario_workspace_invitation: Option<(EmailAddress, workspace_invitation::Model)>,
    previous_workspace_invitation_accept_tokens: HashMap<EmailAddress, Uuid>,
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

    /// Sometimes we need to use the `ServerCtl` from "When" steps,
    /// to avoid rewriting all of its logic in tests.
    /// However, using the dummy attached to the Rocket will cause counters to increase
    /// and this could impact "Then" steps.
    /// This method resets the counters.
    fn reset_server_ctl_counts(&self) {
        let server_ctl = self.server_ctl();
        let mut state = server_ctl.state.lock().unwrap();
        state.conf_reload_count = 0;
    }

    async fn server_manager(&self) -> Result<ServerManager, Error> {
        let server_ctl = self.client.rocket().state::<ServerCtl>().unwrap();
        let db = self.db();
        let server_config = Query::server_config(db)
            .await?
            .expect("Server config not initialized");
        Ok(ServerManager::from(UnauthenticatedServerManager::new(
            db,
            &self.config,
            server_ctl,
            server_config,
        )))
    }

    fn uuid_gen(&self) -> &dependencies::Uuid {
        self.client.rocket().state::<dependencies::Uuid>().unwrap()
    }

    fn server_ctl(&self) -> MutexGuard<DummyServerCtl> {
        self.server_ctl.lock().unwrap()
    }

    fn notifier(&self) -> MutexGuard<DummyNotifier> {
        self.notifier.lock().unwrap()
    }

    fn token(&self, user: String) -> String {
        self.members
            .get(&user)
            .expect("User must be created first")
            .1
            .clone()
    }

    fn scenario_workspace_invitation(&self) -> (EmailAddress, workspace_invitation::Model) {
        self.scenario_workspace_invitation
            .as_ref()
            .expect("Current scenario invitation not stored by previous steps")
            .clone()
    }

    fn previous_workspace_invitation_accept_token(&self, email_address: &EmailAddress) -> Uuid {
        self.previous_workspace_invitation_accept_tokens
            .get(email_address)
            .expect("Previous invitation accept not stored in previous steps")
            .clone()
    }

    fn workspace_invitation(&self, email_address: &EmailAddress) -> workspace_invitation::Model {
        self.workspace_invitations
            .get(email_address)
            .expect("Invitation must be created first")
            .clone()
    }
}

impl TestWorld {
    async fn new() -> Self {
        let config = Arc::new(Config::figment());
        let server_ctl = Arc::new(Mutex::new(DummyServerCtl::new(Default::default())));
        let notifier = Arc::new(Mutex::new(DummyNotifier::new(Default::default())));

        Self {
            config: config.clone(),
            server_ctl: server_ctl.clone(),
            notifier: notifier.clone(),
            client: rocket_test_client(config, server_ctl, notifier).await,
            result: None,
            members: HashMap::new(),
            workspace_invitations: HashMap::new(),
            scenario_workspace_invitation: None,
            previous_workspace_invitation_accept_tokens: HashMap::new(),
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
