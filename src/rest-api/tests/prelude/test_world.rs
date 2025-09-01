// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use axum_test::{TestResponse, TestServer};
use cucumber::*;
use secrecy::{ExposeSecret as _, SecretString};
use service::{
    app_config::{AppConfig, CONFIG_FILE_NAME},
    auth::{AuthService, AuthToken, PasswordResetToken},
    dependencies,
    invitations::{Invitation, InvitationService},
    licensing::LicenseService,
    members::{Member, UnauthenticatedMemberService},
    models::EmailAddress,
    network_checks::{NetworkChecker, PodNetworkConfig},
    notifications::{notifier::email::EmailNotification, NotificationService, Notifier},
    pod_version::PodVersionService,
    sea_orm::DatabaseConnection,
    secrets::{LiveSecretsStore, SecretsStore},
    server_config::{ServerConfig, ServerConfigManager},
    workspace::WorkspaceService,
    xmpp::{BareJid, ServerCtl, XmppServiceInner},
};
use uuid::Uuid;

use super::{
    database::{db_conn, run_migrations},
    mocks::*,
};

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub app_config: Arc<RwLock<AppConfig>>,
    pub db: DatabaseConnection,
    pub mock_server_ctl: MockServerCtl,
    pub server_ctl: ServerCtl,
    pub mock_auth_service: MockAuthService,
    pub auth_service: AuthService,
    pub mock_xmpp_service: MockXmppService,
    pub xmpp_service: XmppServiceInner,
    #[cfg(feature = "test")]
    pub mock_license_service: MockLicenseService,
    pub license_service: LicenseService,
    pub mock_email_notifier: MockNotifier<EmailNotification>,
    pub email_notifier: Notifier<EmailNotification>,
    pub mock_secrets_store: MockSecretsStore,
    pub secrets_store: SecretsStore,
    pub mock_network_checker: MockNetworkChecker,
    pub network_checker: NetworkChecker,
    #[allow(unused)]
    pub mock_pod_version_service: MockPodVersionService,
    pub pod_version_service: PodVersionService,
    pub uuid_gen: dependencies::Uuid,
    pub api: Option<TestServer>,
    pub result: Option<TestResponse>,
    /// Map a name to a member and an authorization token.
    pub members: HashMap<String, (Member, AuthToken)>,
    /// Map an email address to an invitation.
    pub workspace_invitations: HashMap<EmailAddress, Invitation>,
    pub scenario_workspace_invitation: Option<(EmailAddress, Invitation)>,
    pub previous_workspace_invitation_accept_tokens: HashMap<EmailAddress, Uuid>,
    pub password_reset_tokens: HashMap<BareJid, Vec<PasswordResetToken>>,
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

    pub fn app_config<'a>(&'a self) -> RwLockReadGuard<'a, AppConfig> {
        self.app_config.read().unwrap()
    }

    pub fn app_config_mut<'a>(&'a self) -> RwLockWriteGuard<'a, AppConfig> {
        self.app_config.write().unwrap()
    }

    /// Sometimes we need to use the `ServerCtl` from "Given" steps,
    /// to avoid rewriting all of its logic in tests.
    /// However, using the mock attached to the API will cause counters to increase
    /// and this could impact "Then" steps.
    /// This method resets the counters.
    pub fn reset_server_ctl_counts(&self) {
        self.server_ctl_state_mut().conf_reload_count = 0;
    }

    pub fn server_config_manager(&self) -> ServerConfigManager {
        ServerConfigManager::new(
            Arc::new(self.db.clone()),
            self.app_config.clone(),
            Arc::new(self.server_ctl.clone()),
        )
    }

    pub fn member_service(&self) -> UnauthenticatedMemberService {
        UnauthenticatedMemberService::new(
            self.server_ctl.clone(),
            self.auth_service.clone(),
            self.license_service.clone(),
            self.xmpp_service.clone(),
        )
    }

    pub async fn workspace_service(&self) -> WorkspaceService {
        let workspace_jid = self.app_config().workspace_jid();
        WorkspaceService::new(
            self.xmpp_service.clone(),
            workspace_jid,
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

    pub async fn server_config(&self) -> anyhow::Result<ServerConfig> {
        use service::server_config;
        let ref dynamic_server_config = server_config::get(self.db()).await?;
        let server_config =
            ServerConfig::with_default_values(dynamic_server_config, &self.app_config());
        Ok(server_config)
    }

    pub fn server_ctl_state(&self) -> MockServerCtlState {
        self.mock_server_ctl.state.read().unwrap().to_owned()
    }

    pub fn server_ctl_state_mut<'a>(&'a self) -> RwLockWriteGuard<'a, MockServerCtlState> {
        self.mock_server_ctl.state.write().unwrap()
    }

    pub fn xmpp_service_state_mut<'a>(&'a self) -> RwLockWriteGuard<'a, MockXmppServiceState> {
        self.mock_xmpp_service.state.write().unwrap()
    }

    pub fn email_notifier_state<'a>(
        &'a self,
    ) -> RwLockReadGuard<'a, MockNotifierState<EmailNotification>> {
        self.mock_email_notifier.state.read().unwrap()
    }

    pub fn email_notifier_state_mut<'a>(
        &'a self,
    ) -> RwLockWriteGuard<'a, MockNotifierState<EmailNotification>> {
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

    pub async fn pod_network_config(&self) -> PodNetworkConfig {
        let ref app_config = self.app_config().clone();
        let server_config = self.server_config().await.expect("Server config missing");
        PodNetworkConfig::new(app_config, server_config.federation_enabled)
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
        let config_path = crate_root.join("tests").join(CONFIG_FILE_NAME);
        let config = AppConfig::from_path(&config_path)
            .expect(&format!("Invalid config file at {}", config_path.display()));

        let mock_server_ctl_state = Arc::new(RwLock::new(MockServerCtlState::default()));
        let mock_server_ctl = MockServerCtl::new(mock_server_ctl_state.clone());
        let mock_xmpp_service = MockXmppService::default();
        let mock_email_notifier = MockNotifier::<EmailNotification>::default();
        let mock_auth_service = MockAuthService::new(Default::default(), mock_server_ctl_state);
        let mock_license_service = MockLicenseService::new(config.server_fqdn());
        let mock_secrets_store = MockSecretsStore::new(LiveSecretsStore::default(), &config);
        let mock_network_checker = MockNetworkChecker::default();
        let mock_pod_version_service = MockPodVersionService::default();

        let uuid_gen = dependencies::Uuid::from_config(&config);

        // Create API XMPP account
        // NOTE: This is done automatically via Prosody, we need to do it by hand here.
        if let Err(err) = mock_server_ctl
            .add_user(
                &config.api_jid(),
                &config.bootstrap.prose_pod_api_xmpp_password,
            )
            .await
        {
            panic!("Could not create API XMPP account: {}", err);
        }

        let db = db_conn(&config.api.databases.main).await;
        // NOTE: We need to run migrations here before they run in the API because we need to perform actions on the database before the API starts (it's not started by default, since we also test the behavior at startup)
        if let Err(err) = run_migrations(&db).await {
            panic!("Could not run migrations in tests: {err}");
        }

        Self {
            app_config: Arc::new(RwLock::new(config.clone())),
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
            license_service: LicenseService::new(Arc::new(mock_license_service.clone())),
            #[cfg(feature = "test")]
            mock_license_service,
            pod_version_service: PodVersionService::new(Arc::new(mock_pod_version_service.clone())),
            mock_pod_version_service,
            email_notifier: Notifier::from(mock_email_notifier.clone()),
            mock_email_notifier,
            secrets_store: SecretsStore::new(Arc::new(mock_secrets_store.clone())),
            mock_secrets_store,
            network_checker: NetworkChecker::new(Arc::new(mock_network_checker.clone())),
            mock_network_checker,
            uuid_gen,
            password_reset_tokens: HashMap::new(),
        }
    }
}
