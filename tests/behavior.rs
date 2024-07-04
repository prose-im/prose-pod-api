// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod cucumber_parameters;
mod prelude;
mod v1;

use self::prelude::*;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use cucumber::{given, then, World};
use cucumber_parameters::HTTPStatus;
use lazy_static::lazy_static;
use log::debug;
use mock_auth_service::MockAuthService;
use mock_notifier::{MockNotifier, MockNotifierState};
use mock_server_ctl::{MockServerCtl, MockServerCtlState};
use mock_xmpp_service::{MockXmppService, MockXmppServiceState};
use prose_pod_api::error::Error;
use prose_pod_api::guards::{Db, ServerManager, UnauthenticatedServerManager};
use regex::Regex;
use rocket::figment::Figment;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use rocket::{Build, Rocket};
use sea_orm_rocket::Database as _;
use serde::Deserialize;
use service::config::Config;
use service::notifier::AnyNotifier;
use service::repositories::{
    EmailAddress, Invitation, Member, ServerConfig, ServerConfigRepository,
};
use service::sea_orm::DatabaseConnection;
use service::{dependencies, JWTKey, JWTService, XmppServiceInner};
use service::{AuthService, ServerCtl};
use tokio::runtime::Handle;
use tokio::task;
use tracing_subscriber::{
    filter::{self, LevelFilter},
    fmt::format::{self, Format},
    layer::{Layer, SubscriberExt as _},
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Run tests and ignore undefined steps
    // TestWorld::run("tests/features").await;

    // Run tests and ignore undefined steps, but show logs
    // NOTE: Needs the "tracing" feature enabled for `cucumber`
    TestWorld::cucumber()
        // .init_tracing()
        .configure_and_init_tracing(
            format::DefaultFields::new(),
            Format::default(),
            |fmt_layer| {
                let mut targets = vec![
                    ("rocket", LevelFilter::ERROR),
                    ("sea_orm_migration", LevelFilter::WARN),
                    ("rocket::server", LevelFilter::WARN),
                ];

                let running_few_scenarios = std::env::args()
                    .collect::<Vec<_>>()
                    .contains(&"--tags".to_owned());
                if running_few_scenarios {
                    targets.append(&mut vec![
                        ("prose_pod_api", LevelFilter::TRACE),
                        ("service", LevelFilter::TRACE),
                    ]);
                } else {
                    targets.append(&mut vec![
                        ("prose_pod_api", LevelFilter::WARN),
                        ("service", LevelFilter::WARN),
                    ]);
                }

                tracing_subscriber::registry().with(
                    filter::Targets::new()
                        .with_targets(targets)
                        .and_then(fmt_layer),
                )
            },
        )
        .run("tests/features")
        .await;

    // Run and fail on undefined steps
    // TestWorld::cucumber()
    //     .fail_on_skipped()
    //     .run_and_exit("tests/features").await;
}

fn test_rocket(
    config: Config,
    server_ctl: Box<MockServerCtl>,
    xmpp_service: Box<MockXmppService>,
    auth_service: Box<MockAuthService>,
    notifier: Box<MockNotifier>,
) -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::figment())
        .merge(("databases.data.url", "sqlite::memory:"))
        .merge(("log_level", "off"))
        .merge(("databases.data.sqlx_logging", false))
        .merge(("databases.data.sql_log_level", "off"));
    prose_pod_api::custom_rocket(
        rocket::custom(figment),
        config,
        ServerCtl::new(server_ctl),
        XmppServiceInner::new(xmpp_service),
    )
    .manage(AuthService::new(auth_service))
    .manage(dependencies::Notifier::from(AnyNotifier::new(notifier)))
}

pub async fn rocket_test_client(
    config: Config,
    server_ctl: Box<MockServerCtl>,
    xmpp_service: Box<MockXmppService>,
    auth_service: Box<MockAuthService>,
    notifier: Box<MockNotifier>,
) -> Client {
    debug!("Creating Rocket test client...");
    Client::tracked(test_rocket(
        config,
        server_ctl,
        xmpp_service,
        auth_service,
        notifier,
    ))
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
    config: Config,
    server_ctl: MockServerCtl,
    auth_service: MockAuthService,
    xmpp_service: MockXmppService,
    notifier: MockNotifier,
    client: Client,
    result: Option<Response>,
    /// Map a name to a member and an authorization token.
    members: HashMap<String, (Member, String)>,
    /// Map an email address to an invitation.
    workspace_invitations: HashMap<EmailAddress, Invitation>,
    scenario_workspace_invitation: Option<(EmailAddress, Invitation)>,
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

    /// Sometimes we need to use the `ServerCtl` from "Given" steps,
    /// to avoid rewriting all of its logic in tests.
    /// However, using the mock attached to the Rocket will cause counters to increase
    /// and this could impact "Then" steps.
    /// This method resets the counters.
    fn reset_server_ctl_counts(&self) {
        self.server_ctl_state_mut().conf_reload_count = 0;
    }

    async fn server_manager(&self) -> Result<ServerManager, Error> {
        let server_ctl = self.client.rocket().state::<ServerCtl>().unwrap();
        let db = self.db();
        let server_config = ServerConfigRepository::get(db)
            .await?
            .expect("Server config not initialized");
        Ok(ServerManager::from(UnauthenticatedServerManager::new(
            db,
            &self.config,
            server_ctl,
            server_config,
        )))
    }

    async fn server_config(&self) -> Result<ServerConfig, Error> {
        Ok(self.server_manager().await?.server_config())
    }

    fn uuid_gen(&self) -> &dependencies::Uuid {
        self.client.rocket().state::<dependencies::Uuid>().unwrap()
    }

    fn server_ctl_state(&self) -> RwLockReadGuard<MockServerCtlState> {
        self.server_ctl.state.read().unwrap()
    }

    fn server_ctl_state_mut(&self) -> RwLockWriteGuard<MockServerCtlState> {
        self.server_ctl.state.write().unwrap()
    }

    fn xmpp_service_state_mut(&self) -> RwLockWriteGuard<MockXmppServiceState> {
        self.xmpp_service.state.write().unwrap()
    }

    fn notifier_state(&self) -> RwLockReadGuard<MockNotifierState> {
        self.notifier.state.read().unwrap()
    }

    fn token(&self, user: String) -> String {
        self.members
            .get(&user)
            .expect("User must be created first")
            .1
            .clone()
    }

    fn scenario_workspace_invitation(&self) -> (EmailAddress, Invitation) {
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

    fn workspace_invitation(&self, email_address: &EmailAddress) -> Invitation {
        self.workspace_invitations
            .get(email_address)
            .expect("Invitation must be created first")
            .clone()
    }
}

impl TestWorld {
    async fn new() -> Self {
        let config = Config::figment();
        let mock_server_ctl_state = Arc::new(RwLock::new(MockServerCtlState::default()));
        let mock_server_ctl = MockServerCtl::new(mock_server_ctl_state.clone());
        let mock_xmpp_service = MockXmppService::default();
        let mock_notifier = MockNotifier::default();
        let jwt_service = JWTService::new(JWTKey::custom("test_key"));
        let mock_auth_service =
            MockAuthService::new(jwt_service, Default::default(), mock_server_ctl_state);

        Self {
            config: config.clone(),
            client: rocket_test_client(
                config,
                Box::new(mock_server_ctl.clone()),
                Box::new(mock_xmpp_service.clone()),
                Box::new(mock_auth_service.clone()),
                Box::new(mock_notifier.clone()),
            )
            .await,
            result: None,
            members: HashMap::new(),
            workspace_invitations: HashMap::new(),
            scenario_workspace_invitation: None,
            previous_workspace_invitation_accept_tokens: HashMap::new(),
            server_ctl: mock_server_ctl,
            xmpp_service: mock_xmpp_service,
            auth_service: mock_auth_service,
            notifier: mock_notifier,
        }
    }
}

#[given("the XMPP server is offline")]
fn given_xmpp_server_offline(world: &mut TestWorld) {
    world.xmpp_service_state_mut().online = false;
    world.server_ctl_state_mut().online = false;
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

#[then("the response is a SSE stream")]
fn then_response_is_sse_stream(world: &mut TestWorld) {
    let res = world.result();
    assert_eq!(res.content_type, Some(ContentType::EventStream));
}

lazy_static! {
    static ref UNEXPECTED_SEMICOLON_REGEX: Regex = Regex::new(r"\n:(\n|$)").unwrap();
}

#[then(expr = "one SSE event is {string}")]
async fn then_sse_event(world: &mut TestWorld, value: String) {
    let res = world.result();
    let events = res
        .body()
        .split("\n\n")
        // Fix random "\n:" inconsistently added by Rocket for no apparent reason
        .map(|s| UNEXPECTED_SEMICOLON_REGEX.replace_all(s, "$1").to_string())
        .collect::<Vec<String>>();
    let expected = value
        // Unescape double quotes
        .replace(r#"\""#, r#"""#)
        // Unescape newlines
        .replace("\\n", "\n");
    assert!(
        events.contains(&expected),
        "events: {events:?}\nexpected: {expected:?}"
    );
}
