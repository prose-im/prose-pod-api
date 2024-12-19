// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod cucumber_parameters;
mod features;
mod prelude;

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use cucumber::{given, then, when, World};
use cucumber_parameters::{HTTPStatus, JID};
use lazy_static::lazy_static;
use mock_auth_service::MockAuthService;
use mock_network_checker::MockNetworkChecker;
use mock_notifier::{MockNotifier, MockNotifierState};
use mock_secrets_store::MockSecretsStore;
use mock_server_ctl::{MockServerCtl, MockServerCtlState};
use mock_xmpp_service::{MockXmppService, MockXmppServiceState};
use prose_pod_api::guards::Db;
use regex::Regex;
use rocket::{
    figment::{providers::Serialized, Figment},
    http::{ContentType, Status},
    local::asynchronous::{Client, LocalResponse},
};
use sea_orm_rocket::Database;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use service::{
    auth::{AuthService, AuthToken},
    dependencies,
    init::InitController,
    invitations::Invitation,
    members::{Member, UserService},
    models::EmailAddress,
    network_checks::NetworkChecker,
    notifications::dependencies::{any_notifier::AnyNotifier, Notifier},
    sea_orm::DatabaseConnection,
    secrets::{LiveSecretsStore, SecretsStore, SecretsStoreImpl},
    server_config::{entities::server_config, ServerConfig, ServerConfigRepository},
    workspace::WorkspaceController,
    xmpp::{ServerCtl, ServerCtlImpl as _, ServerManager, XmppServiceInner},
    AppConfig,
};
use service::{errors::DbErr, MigratorTrait as _};
use tokio::{runtime::Handle, task};
use tracing::debug;
use tracing_subscriber::{
    filter::{self, LevelFilter},
    fmt::format::{self, Format},
    layer::{Layer, SubscriberExt as _},
};
use uuid::Uuid;

use self::prelude::*;

#[tokio::main]
async fn main() {
    TestWorld::cucumber()
        // .init_tracing()
        .configure_and_init_tracing(
            format::DefaultFields::new(),
            Format::default(),
            |fmt_layer| {
                let targets = vec![
                    ("rocket", LevelFilter::ERROR),
                    ("sea_orm_migration", LevelFilter::WARN),
                    ("rocket::server", LevelFilter::WARN),
                    ("globset", LevelFilter::WARN),
                ];

                let args = std::env::args().collect::<Vec<_>>();
                let running_few_scenarios = args.contains(&"@testing".to_owned());
                let debug = args.contains(&"@debug".to_owned());

                let default_level = if running_few_scenarios || debug {
                    LevelFilter::TRACE
                } else {
                    LevelFilter::WARN
                };

                tracing_subscriber::registry().with(
                    filter::Targets::new()
                        .with_default(default_level)
                        .with_targets(targets)
                        .and_then(fmt_layer),
                )
            },
        )
        // Fail on undefined steps
        // .fail_on_skipped()
        .run_and_exit("../../features")
        .await;
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
    app_config: AppConfig,
    rocket_config_provider: Figment,
    db: Db,
    mock_server_ctl: MockServerCtl,
    server_ctl: ServerCtl,
    mock_auth_service: MockAuthService,
    auth_service: AuthService,
    mock_xmpp_service: MockXmppService,
    xmpp_service: XmppServiceInner,
    mock_notifier: MockNotifier,
    notifier: Notifier,
    mock_secrets_store: MockSecretsStore,
    secrets_store: SecretsStore,
    mock_network_checker: MockNetworkChecker,
    network_checker: NetworkChecker,
    uuid_gen: dependencies::Uuid,
    client: Option<Client>,
    result: Option<Response>,
    /// Map a name to a member and an authorization token.
    members: HashMap<String, (Member, AuthToken)>,
    /// Map an email address to an invitation.
    workspace_invitations: HashMap<EmailAddress, Invitation>,
    scenario_workspace_invitation: Option<(EmailAddress, Invitation)>,
    previous_workspace_invitation_accept_tokens: HashMap<EmailAddress, Uuid>,
}

impl TestWorld {
    async fn create_rocket_test_client(&self) -> Client {
        debug!("Creating Rocket test client…");

        let rocket = rocket::custom(self.rocket_config_provider.clone())
            // NOTE: `Db::clone` returns a database pool (`SeaOrmPool`), not a `Db`!
            .manage(Db::from(self.db.clone()));

        Client::tracked(prose_pod_api::custom_rocket(
            rocket,
            self.app_config.clone(),
            self.server_ctl.clone(),
            self.xmpp_service.clone(),
            self.auth_service.clone(),
            self.notifier.clone(),
            self.secrets_store.clone(),
            self.network_checker.clone(),
        ))
        .await
        .expect("valid rocket instance")
    }

    fn client(&self) -> &Client {
        self.client
            .as_ref()
            .expect("The Prose Pod API must be started with 'Given the Prose Pod API has started'")
    }

    fn result(&mut self) -> &mut Response {
        match &mut self.result {
            Some(res) => res,
            None => panic!("A call must be made before"),
        }
    }

    fn db(&self) -> &DatabaseConnection {
        &self.db.conn
    }

    /// Sometimes we need to use the `ServerCtl` from "Given" steps,
    /// to avoid rewriting all of its logic in tests.
    /// However, using the mock attached to the Rocket will cause counters to increase
    /// and this could impact "Then" steps.
    /// This method resets the counters.
    fn reset_server_ctl_counts(&self) {
        self.server_ctl_state_mut().conf_reload_count = 0;
    }

    async fn server_manager(&self) -> Result<ServerManager, DbErr> {
        let server_config = self.server_config_model().await?;
        Ok(ServerManager::new(
            Arc::new(self.db().clone()),
            Arc::new(self.app_config.clone()),
            Arc::new(self.server_ctl.clone()),
            server_config,
        ))
    }

    fn user_service(&self) -> UserService {
        UserService::new(
            Arc::new(self.server_ctl.clone()),
            Arc::new(self.auth_service.clone()),
            Arc::new(self.xmpp_service.clone()),
        )
    }

    fn init_controller(&self) -> InitController {
        let db = self.db();
        InitController {
            db: Arc::new(db.clone()),
        }
    }

    async fn workspace_controller(&self) -> WorkspaceController {
        WorkspaceController::new(
            Arc::new(self.db().clone()),
            Arc::new(self.xmpp_service.clone()),
            Arc::new(self.app_config.clone()),
            &self
                .server_config()
                .await
                .expect("Server config not initialized"),
            Arc::new(self.secrets_store.clone()),
        )
        .expect("Workspace not initialized")
    }

    async fn server_config_model(&self) -> Result<server_config::Model, DbErr> {
        let db = self.db();
        Ok(ServerConfigRepository::get(db)
            .await?
            .expect("Server config not initialized"))
    }

    async fn server_config(&self) -> Result<ServerConfig, DbErr> {
        let model = self.server_config_model().await?;
        Ok(model.with_default_values_from(&self.app_config))
    }

    fn server_ctl_state(&self) -> MockServerCtlState {
        self.mock_server_ctl.state.read().unwrap().to_owned()
    }

    fn server_ctl_state_mut(&self) -> RwLockWriteGuard<MockServerCtlState> {
        self.mock_server_ctl.state.write().unwrap()
    }

    fn xmpp_service_state_mut(&self) -> RwLockWriteGuard<MockXmppServiceState> {
        self.mock_xmpp_service.state.write().unwrap()
    }

    fn notifier_state(&self) -> MockNotifierState {
        self.mock_notifier.state.read().unwrap().to_owned()
    }

    fn token(&self, user: String) -> SecretString {
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
        // NOTE: Behavior tests don't need to read the environment, therefore we have to set the required variables.
        let api_xmpp_password = SecretString::from_str("anything").unwrap();
        std::env::set_var(
            "PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD",
            &api_xmpp_password.expose_secret(),
        );
        let config = AppConfig::figment();

        let mock_server_ctl_state = Arc::new(RwLock::new(MockServerCtlState::default()));
        let mock_server_ctl = MockServerCtl::new(mock_server_ctl_state.clone());
        let mock_xmpp_service = MockXmppService::default();
        let mock_notifier = MockNotifier::default();
        let mock_auth_service = MockAuthService::new(Default::default(), mock_server_ctl_state);
        let mock_secrets_store =
            MockSecretsStore::new(LiveSecretsStore::from_config(&config), &config);
        let mock_network_checker = MockNetworkChecker::default();

        let uuid_gen = dependencies::Uuid::from_config(&config);

        // Create API XMPP account
        // NOTE: This is done automatically via Prosody, we need to do it by hand here.
        if let Err(err) = mock_server_ctl
            .add_user(
                &config.api_jid(),
                &mock_secrets_store.prose_pod_api_xmpp_password(),
            )
            .await
        {
            panic!("Could not create API XMPP account: {}", err);
        }

        let rocket_config_provider = rocket::Config::figment()
            .merge(("databases.data.url", "sqlite::memory:"))
            .merge(("log_level", "off"))
            .merge(("databases.data.sqlx_logging", false))
            .merge(("databases.data.sql_log_level", "off"));

        let pool = db_pool(&rocket_config_provider).await;
        let db = Db::from(pool);
        if let Err(err) = run_migrations(&db.conn).await {
            panic!("Could not run migrations in tests: {err}");
        }

        Self {
            app_config: config.clone(),
            rocket_config_provider,
            db,
            client: None,
            result: None,
            members: HashMap::new(),
            workspace_invitations: HashMap::new(),
            scenario_workspace_invitation: None,
            previous_workspace_invitation_accept_tokens: HashMap::new(),
            server_ctl: ServerCtl::new(Arc::new(mock_server_ctl.clone())),
            mock_server_ctl,
            xmpp_service: XmppServiceInner::new(Arc::new(mock_xmpp_service.clone())),
            mock_xmpp_service,
            auth_service: AuthService::new(Arc::new(mock_auth_service.clone())),
            mock_auth_service,
            notifier: Notifier::from(AnyNotifier::new(Box::new(mock_notifier.clone()))),
            mock_notifier,
            secrets_store: SecretsStore::new(Arc::new(mock_secrets_store.clone())),
            mock_secrets_store,
            network_checker: NetworkChecker::new(Arc::new(mock_network_checker.clone())),
            mock_network_checker,
            uuid_gen,
        }
    }
}

// NOTE: Logic and values come from <https://github.com/SeaQL/sea-orm/blob/825cb14e39ef859af529263a39c0aff8b38b8082/sea-orm-rocket/lib/src/database.rs#L234-L254>.
async fn db_pool(rocket_config_provider: &Figment) -> <Db as sea_orm_rocket::Database>::Pool {
    let workers: usize = rocket_config_provider
        .extract_inner(rocket::Config::WORKERS)
        .unwrap_or_else(|_| rocket::Config::default().workers);

    let figment = rocket_config_provider
        .focus(&format!("databases.{}", Db::NAME))
        .merge(Serialized::default("max_connections", workers * 4))
        .merge(Serialized::default("connect_timeout", 5))
        .merge(Serialized::default("sqlx_logging", true));

    <<Db as sea_orm_rocket::Database>::Pool as sea_orm_rocket::Pool>::init(&figment)
        .await
        .expect("Invalid database configuration")
}

async fn run_migrations(conn: &DatabaseConnection) -> Result<(), DbErr> {
    debug!("Running database migrations before creating the Rocket…");
    service::Migrator::up(conn, None).await
}

#[given("the Prose Pod API has started")]
async fn given_api_started(world: &mut TestWorld) {
    assert!(world.client.is_none());
    world.client = Some(world.create_rocket_test_client().await);
    world.reset_server_ctl_counts();
}

#[when("the Prose Pod API starts")]
async fn when_api_starts(world: &mut TestWorld) {
    assert!(world.client.is_none());
    world.client = Some(world.create_rocket_test_client().await);
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
    static ref UNEXPECTED_SEMICOLON_REGEX: Regex = Regex::new(r"(\n|^):(\n|$)").unwrap();
    static ref UNEXPECTED_NEWLINE_REGEX: Regex = Regex::new(r"\n$").unwrap();
}

#[then(expr = "one SSE event is {string}")]
async fn then_sse_event(world: &mut TestWorld, value: String) {
    let res = world.result();
    let events = res
        .body()
        .split("\n\n")
        // Fix random "\n:" inconsistently added by Rocket for no apparent reason
        .map(|s| UNEXPECTED_SEMICOLON_REGEX.replace_all(s, "$1"))
        // Fix random "\n" inconsistently added by Rocket for no apparent reason
        .map(|s| UNEXPECTED_NEWLINE_REGEX.replace_all(&s, "").to_string())
        .collect::<Vec<String>>();
    let expected = value
        // Unescape double quotes
        .replace(r#"\""#, r#"""#)
        // Unescape newlines
        .replace("\\n", "\n");
    assert!(
        events.contains(&expected),
        "events: {events:#?}\nexpected: {expected:?}"
    );
}

#[then(expr = "<{jid}>'s password is changed")]
fn then_password_changed(world: &mut TestWorld, jid: JID) {
    assert_ne!(world.mock_secrets_store.changes_count(&jid), 0);
}
