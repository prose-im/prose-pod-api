// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::HashMap,
    path::Path,
    str::FromStr,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use axum_test::{TestResponse, TestServer};
use cucumber::*;
use secrecy::{ExposeSecret as _, SecretString};
use service::{
    app_config::{AppConfig, CONFIG_FILE_NAME},
    auth::{AuthService, AuthToken},
    dependencies,
    errors::DbErr,
    init::InitService,
    invitations::{Invitation, InvitationService},
    members::{Member, UnauthenticatedMemberService},
    models::EmailAddress,
    network_checks::NetworkChecker,
    notifications::{notifier::email::EmailNotification, NotificationService, Notifier},
    sea_orm::DatabaseConnection,
    secrets::{LiveSecretsStore, SecretsStore},
    server_config::{entities::server_config, ServerConfig, ServerConfigRepository},
    workspace::WorkspaceService,
    xmpp::{JidDomain, ServerCtl, ServerManager, XmppServiceInner},
};
use uuid::Uuid;

use super::{
    database::{db_conn, run_migrations},
    mocks::*,
};

pub const DEFAULT_DOMAIN: &'static str = "prose.test.org";

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub app_config: AppConfig,
    pub db: DatabaseConnection,
    pub mock_server_ctl: MockServerCtl,
    pub server_ctl: ServerCtl,
    pub mock_auth_service: MockAuthService,
    pub auth_service: AuthService,
    pub mock_xmpp_service: MockXmppService,
    pub xmpp_service: XmppServiceInner,
    pub mock_email_notifier: MockNotifier<EmailNotification>,
    pub email_notifier: Notifier<EmailNotification>,
    pub mock_secrets_store: MockSecretsStore,
    pub secrets_store: SecretsStore,
    pub mock_network_checker: MockNetworkChecker,
    pub network_checker: NetworkChecker,
    pub uuid_gen: dependencies::Uuid,
    pub api: Option<TestServer>,
    pub result: Option<TestResponse>,
    /// Map a name to a member and an authorization token.
    pub members: HashMap<String, (Member, AuthToken)>,
    /// Map an email address to an invitation.
    pub workspace_invitations: HashMap<EmailAddress, Invitation>,
    pub scenario_workspace_invitation: Option<(EmailAddress, Invitation)>,
    pub previous_workspace_invitation_accept_tokens: HashMap<EmailAddress, Uuid>,
    pub initial_server_domain: JidDomain,
}

impl TestWorld {
    pub fn api(&self) -> &TestServer {
        self.api
            .as_ref()
            .expect("The Prose Pod API must be started with 'Given the Prose Pod API has started'")
    }

    pub fn result(&mut self) -> &mut TestResponse {
        match &mut self.result {
            Some(res) => res,
            None => panic!("A call must be made before"),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Sometimes we need to use the `ServerCtl` from "Given" steps,
    /// to avoid rewriting all of its logic in tests.
    /// However, using the mock attached to the API will cause counters to increase
    /// and this could impact "Then" steps.
    /// This method resets the counters.
    pub fn reset_server_ctl_counts(&self) {
        self.server_ctl_state_mut().conf_reload_count = 0;
    }

    pub async fn server_manager(&self) -> Result<Option<ServerManager>, DbErr> {
        let Some(server_config) = self.opt_server_config_model().await? else {
            return Ok(None);
        };
        Ok(Some(ServerManager::new(
            Arc::new(self.db().clone()),
            Arc::new(self.app_config.clone()),
            Arc::new(self.server_ctl.clone()),
            server_config,
        )))
    }

    pub fn member_service(&self) -> UnauthenticatedMemberService {
        UnauthenticatedMemberService::new(
            Arc::new(self.server_ctl.clone()),
            Arc::new(self.auth_service.clone()),
            Arc::new(self.xmpp_service.clone()),
        )
    }

    pub fn init_service(&self) -> InitService {
        let db = self.db();
        InitService {
            db: Arc::new(db.clone()),
        }
    }

    pub async fn workspace_service(&self) -> WorkspaceService {
        WorkspaceService::new(
            Arc::new(self.xmpp_service.clone()),
            Arc::new(self.app_config.clone()),
            &(self.server_config().await)
                .expect("Error getting server config")
                .domain,
            Arc::new(self.secrets_store.clone()),
        )
        .expect("Workspace not initialized")
    }

    pub fn invitation_service(&self) -> InvitationService {
        InvitationService::new(
            self.db().clone(),
            self.uuid_gen.clone(),
            self.member_service(),
        )
    }

    pub fn notifcation_service(&self) -> NotificationService {
        NotificationService::new(self.email_notifier.clone())
    }

    pub async fn opt_server_config_model(&self) -> Result<Option<server_config::Model>, DbErr> {
        let db = self.db();
        Ok(ServerConfigRepository::get(db).await?)
    }

    pub async fn server_config_model(&self) -> Result<server_config::Model, DbErr> {
        self.opt_server_config_model()
            .await
            .map(|opt| opt.expect("Server config not initialized."))
    }

    pub async fn opt_server_config(&self) -> Result<Option<ServerConfig>, DbErr> {
        let config = self.opt_server_config_model().await?;
        Ok(config.map(|model| model.with_default_values_from(&self.app_config)))
    }

    pub async fn server_config(&self) -> Result<ServerConfig, DbErr> {
        self.opt_server_config()
            .await
            .map(|opt| opt.expect("Server config not initialized."))
    }

    pub fn server_ctl_state(&self) -> MockServerCtlState {
        self.mock_server_ctl.state.read().unwrap().to_owned()
    }

    pub fn server_ctl_state_mut(&self) -> RwLockWriteGuard<MockServerCtlState> {
        self.mock_server_ctl.state.write().unwrap()
    }

    pub fn xmpp_service_state_mut(&self) -> RwLockWriteGuard<MockXmppServiceState> {
        self.mock_xmpp_service.state.write().unwrap()
    }

    pub fn email_notifier_state(&self) -> RwLockReadGuard<MockNotifierState<EmailNotification>> {
        self.mock_email_notifier.state.read().unwrap()
    }

    pub fn email_notifier_state_mut(
        &self,
    ) -> RwLockWriteGuard<MockNotifierState<EmailNotification>> {
        self.mock_email_notifier.state.write().unwrap()
    }

    pub fn token(&self, user: &str) -> SecretString {
        self.members
            .get(user)
            .expect("User must be created first")
            .1
            .clone()
    }

    pub fn scenario_workspace_invitation(&self) -> (EmailAddress, Invitation) {
        self.scenario_workspace_invitation
            .as_ref()
            .expect("Current scenario invitation not stored by previous steps")
            .clone()
    }

    pub fn previous_workspace_invitation_accept_token(&self, email_address: &EmailAddress) -> Uuid {
        self.previous_workspace_invitation_accept_tokens
            .get(email_address)
            .expect("Previous invitation accept not stored in previous steps")
            .clone()
    }

    pub fn workspace_invitation(&self, email_address: &EmailAddress) -> Invitation {
        self.workspace_invitations
            .get(email_address)
            .expect("Invitation must be created first")
            .clone()
    }
}

impl TestWorld {
    async fn new() -> Self {
        // NOTE: Behavior tests don't need to read the environment, therefore we have to set the required variables.
        let api_xmpp_password = SecretString::from("anything");
        std::env::set_var(
            "PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD",
            &api_xmpp_password.expose_secret(),
        );
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config = AppConfig::from_path(crate_root.join("tests").join(CONFIG_FILE_NAME));

        let mock_server_ctl_state = Arc::new(RwLock::new(MockServerCtlState::default()));
        let mock_server_ctl = MockServerCtl::new(mock_server_ctl_state.clone());
        let mock_xmpp_service = MockXmppService::default();
        let mock_email_notifier = MockNotifier::<EmailNotification>::default();
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

        let db = db_conn(&config.databases.main).await;
        // NOTE: We need to run migrations here before they run in the API because we need to perform actions on the database before the API starts (it's not started by default, since we also test the behavior at startup)
        if let Err(err) = run_migrations(&db).await {
            panic!("Could not run migrations in tests: {err}");
        }

        Self {
            app_config: config.clone(),
            db,
            api: None,
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
            email_notifier: Notifier::from(mock_email_notifier.clone()),
            mock_email_notifier,
            secrets_store: SecretsStore::new(Arc::new(mock_secrets_store.clone())),
            mock_secrets_store,
            network_checker: NetworkChecker::new(Arc::new(mock_network_checker.clone())),
            mock_network_checker,
            uuid_gen,
            initial_server_domain: JidDomain::from_str(DEFAULT_DOMAIN).unwrap(),
        }
    }
}
