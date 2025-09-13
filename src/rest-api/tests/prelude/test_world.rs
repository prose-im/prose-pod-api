// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use axum_test::{TestResponse, TestServer};
use cucumber::*;
use figment::Figment;
use secrecy::{ExposeSecret as _, SecretString};
use service::{
    app_config::{AppConfig, CONFIG_FILE_NAME},
    auth::{AuthService, AuthToken, PasswordResetToken},
    dependencies,
    factory_reset::FactoryResetService,
    invitations::{Invitation, InvitationService},
    licensing::LicenseService,
    members::{Member, UnauthenticatedMemberService},
    models::EmailAddress,
    network_checks::{NetworkChecker, PodNetworkConfig},
    notifications::{
        notifier::{email::EmailNotification, GenericNotifier},
        NotificationService, Notifier,
    },
    pod_version::PodVersionService,
    sea_orm::DatabaseConnection,
    secrets::SecretsStore,
    server_config::{ServerConfig, ServerConfigManager},
    workspace::WorkspaceService,
    xmpp::{BareJid, ServerCtl, XmppServiceInner},
};
use uuid::Uuid;

use crate::prelude::steps::app_config::reload_config;

use super::{
    database::{db_conn, run_migrations},
    mocks::*,
};

lazy_static::lazy_static! {
    pub(crate) static ref CRATE_ROOT: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    pub(crate) static ref CONFIG_PATH: PathBuf = CRATE_ROOT.join("tests").join(CONFIG_FILE_NAME);
}

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub app_config: Option<Arc<AppConfig>>,
    pub db: DatabaseConnection,
    pub mock_server_ctl: Arc<MockServerCtl>,
    pub server_ctl: ServerCtl,
    pub mock_auth_service: Arc<MockAuthService>,
    pub auth_service: AuthService,
    pub mock_xmpp_service: Arc<MockXmppService>,
    pub xmpp_service: XmppServiceInner,
    #[cfg(feature = "test")]
    pub mock_license_service: Option<Arc<MockLicenseService>>,
    pub license_service: Option<LicenseService>,
    pub mock_email_notifier: Arc<MockNotifier<EmailNotification>>,
    pub email_notifier: Notifier<EmailNotification>,
    pub mock_secrets_store: Option<Arc<MockSecretsStore>>,
    pub secrets_store: Option<SecretsStore>,
    pub mock_network_checker: Arc<MockNetworkChecker>,
    pub network_checker: NetworkChecker,
    #[allow(unused)]
    pub mock_pod_version_service: Arc<MockPodVersionService>,
    pub pod_version_service: PodVersionService,
    pub factory_reset_service: FactoryResetService,
    pub uuid_gen: Option<dependencies::Uuid>,
    pub result: Option<TestResponse>,
    /// Map a name to a member and an authorization token.
    pub members: HashMap<String, (Member, AuthToken)>,
    /// Map an email address to an invitation.
    pub workspace_invitations: HashMap<EmailAddress, Invitation>,
    pub scenario_workspace_invitation: Option<(EmailAddress, Invitation)>,
    pub previous_workspace_invitation_accept_tokens: HashMap<EmailAddress, Uuid>,
    pub password_reset_tokens: HashMap<BareJid, Vec<PasswordResetToken>>,

    pub api: Option<TestServer>,
    pub config_overrides: Figment,
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

    pub fn app_config(&self) -> Arc<AppConfig> {
        self.app_config
            .as_ref()
            .expect("app_config not initialized")
            .clone()
    }
    pub fn mock_license_service(&self) -> &MockLicenseService {
        self.mock_license_service
            .as_ref()
            .expect("mock_license_service not initialized")
    }
    pub fn license_service(&self) -> &LicenseService {
        self.license_service
            .as_ref()
            .expect("license_service not initialized")
    }
    pub fn mock_secrets_store(&self) -> &MockSecretsStore {
        self.mock_secrets_store
            .as_ref()
            .expect("mock_secrets_store not initialized")
    }
    pub fn secrets_store(&self) -> &SecretsStore {
        self.secrets_store
            .as_ref()
            .expect("secrets_store not initialized")
    }
    pub fn uuid_gen(&self) -> &dependencies::Uuid {
        self.uuid_gen.as_ref().expect("uuid_gen not initialized")
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
            self.app_config().clone(),
            Arc::new(self.server_ctl.clone()),
        )
    }

    pub fn member_service(&self) -> UnauthenticatedMemberService {
        UnauthenticatedMemberService::new(
            self.server_ctl.clone(),
            self.auth_service.clone(),
            self.license_service().clone(),
            self.xmpp_service.clone(),
        )
    }

    pub async fn workspace_service(&self) -> WorkspaceService {
        let workspace_jid = self.app_config().workspace_jid();
        WorkspaceService::new(
            self.xmpp_service.clone(),
            workspace_jid,
            Arc::new(self.secrets_store().clone()),
        )
        .expect("Workspace not initialized")
    }

    pub fn invitation_service(&self) -> InvitationService {
        InvitationService::new(
            self.db().clone(),
            self.uuid_gen().clone(),
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

    pub fn server_ctl_state(&self) -> RwLockReadGuard<'_, MockServerCtlState> {
        self.mock_server_ctl.state.read().unwrap()
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
        let ref app_config = self.app_config();
        let server_config = self.server_config().await.expect("Server config missing");
        PodNetworkConfig::new(&app_config, server_config.federation_enabled)
    }

    pub fn set_config(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        use figment::providers::Serialized;

        let mut figment = self.config_overrides.clone();

        let json: serde_json::Value = serde_json::from_str(value)?;
        figment = figment.merge(Serialized::default(key, json));

        self.config_overrides = figment;

        Ok(())
    }

    pub fn unset_config(&mut self, key: &str) {
        use figment::providers::Serialized;

        let mut figment = self.config_overrides.clone();

        let json = serde_json::Value::Null;
        figment = figment.merge(Serialized::default(key, json));

        self.config_overrides = figment;
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

        let db = {
            let config = AppConfig::from_path(CONFIG_PATH.as_path())
                .expect(&format!("Invalid config file at {}", CONFIG_PATH.display()));

            db_conn(&config.api.databases.main).await
        };

        // NOTE: We need to run migrations here before they run in the API because we need to perform actions on the database before the API starts (it's not started by default, since we also test the behavior at startup)
        if let Err(err) = run_migrations(&db).await {
            panic!("Could not run migrations in tests: {err}");
        }

        let mock_server_ctl_state = Arc::new(RwLock::new(MockServerCtlState::default()));
        let mock_server_ctl = Arc::new(MockServerCtl::new(
            mock_server_ctl_state.clone(),
            db.clone(),
        ));
        let mock_xmpp_service = Arc::new(MockXmppService::default());
        let mock_email_notifier = Arc::new(MockNotifier::<EmailNotification>::default());
        let mock_auth_service = Arc::new(MockAuthService::new(
            Default::default(),
            mock_server_ctl_state,
        ));
        let mock_network_checker = Arc::new(MockNetworkChecker::default());
        let mock_pod_version_service = Arc::new(MockPodVersionService::default());

        let mut world = Self {
            app_config: None,
            db,
            api: None,
            config_overrides: Figment::new(),
            result: None,
            members: HashMap::new(),
            workspace_invitations: HashMap::new(),
            scenario_workspace_invitation: None,
            previous_workspace_invitation_accept_tokens: HashMap::new(),
            server_ctl: ServerCtl::new(mock_server_ctl.clone()),
            mock_server_ctl,
            xmpp_service: XmppServiceInner::new(mock_xmpp_service.clone()),
            mock_xmpp_service,
            auth_service: AuthService::new(mock_auth_service.clone()),
            mock_auth_service,
            license_service: None,
            #[cfg(feature = "test")]
            mock_license_service: None,
            pod_version_service: PodVersionService::new(mock_pod_version_service.clone()),
            mock_pod_version_service,
            factory_reset_service: FactoryResetService::default(),
            email_notifier: Notifier::from(mock_email_notifier.clone()
                as Arc<dyn GenericNotifier<Notification = EmailNotification>>),
            mock_email_notifier,
            secrets_store: None,
            mock_secrets_store: None,
            network_checker: NetworkChecker::new(mock_network_checker.clone()),
            mock_network_checker,
            uuid_gen: None,
            password_reset_tokens: HashMap::new(),
        };

        reload_config(&mut world);

        world
    }
}
