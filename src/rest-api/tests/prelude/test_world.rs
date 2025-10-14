// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use axum_test::{TestResponse, TestServer};
use cucumber::*;
use figment::Figment;
use service::{
    app_config::{AppConfig, CONFIG_FILE_NAME},
    auth::{AuthService, AuthToken},
    factory_reset::FactoryResetService,
    identity_provider::{IdentityProvider, VcardIdentityProvider},
    invitations::{
        invitation_service::InvitationApplicationService, Invitation, InvitationRepository,
        InvitationService,
    },
    licensing::LicensingService,
    members::{MemberService, UserApplicationService, UserRepository},
    models::{DatabaseRwConnectionPools, EmailAddress},
    network_checks::{NetworkChecker, PodNetworkConfig},
    notifications::{
        notifier::{email::EmailNotification, GenericNotifier},
        NotificationService, Notifier,
    },
    pod_version::PodVersionService,
    prose_pod_server_service::ProsePodServerService,
    secrets_store::SecretsStore,
    server_config::{ServerConfig, ServerConfigManager},
    util::random_string_alphanumeric,
    workspace::{Workspace, WorkspaceService},
    xmpp::XmppService,
};

use crate::prelude::util::{name_to_jid, user_missing};

use super::{
    app_config::reload_config,
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
    pub db: DatabaseRwConnectionPools,
    pub user_repository: Option<UserRepository>,
    pub mock_user_repository: Option<Arc<MockUserRepository>>,
    pub invitation_repository: Option<InvitationRepository>,
    pub mock_invitation_repository: Option<Arc<MockInvitationRepository>>,
    pub auth_service: Option<AuthService>,
    pub mock_auth_service: Option<Arc<MockAuthService>>,
    pub xmpp_service: Option<XmppService>,
    pub mock_xmpp_service: Option<Arc<MockXmppService>>,
    pub workspace_service: Option<WorkspaceService>,
    pub mock_workspace_service: Option<Arc<MockWorkspaceService>>,
    pub licensing_service: Option<LicensingService>,
    pub mock_licensing_service: Option<Arc<MockLicensingService>>,
    pub email_notifier: Notifier<EmailNotification>,
    pub mock_email_notifier: Arc<MockNotifier<EmailNotification>>,
    pub secrets_store: Option<SecretsStore>,
    pub mock_secrets_store: Option<Arc<MockSecretsStore>>,
    pub network_checker: NetworkChecker,
    pub mock_network_checker: Arc<MockNetworkChecker>,
    #[allow(unused)]
    pub mock_pod_version_service: Arc<MockPodVersionService>,
    pub pod_version_service: PodVersionService,
    pub factory_reset_service: FactoryResetService,
    pub server_service: Option<ProsePodServerService>,
    pub mock_server_service: Option<Arc<MockServerService>>,
    pub user_application_service: Option<UserApplicationService>,
    pub mock_user_application_service: Option<Arc<MockUserService>>,
    #[allow(unused)]
    pub mock_invitation_application_service: Option<MockInvitationService>,
    pub identity_provider: IdentityProvider,
    #[allow(unused)]
    pub mock_identity_provider: Arc<MockIdentityProvider>,
    pub result: Option<TestResponse>,
    pub scenario_workspace_invitation: Option<(EmailAddress, Invitation)>,

    pub mock_server_state: Arc<RwLock<MockServerServiceState>>,
    pub mock_auth_service_state: Arc<RwLock<MockAuthServiceState>>,
    pub mock_xmpp_service_state: Arc<RwLock<MockXmppServiceState>>,
    pub mock_workspace_service_state: Option<Arc<RwLock<MockWorkspaceServiceState>>>,

    pub api: Option<TestServer>,
    pub config_overrides: Figment,
}

impl TestWorld {
    /// Sometimes we need to use the `ServerCtl` from "Given" steps,
    /// to avoid rewriting all of its logic in tests.
    /// However, using the mock attached to the API will cause counters to increase
    /// and this could impact "Then" steps.
    /// This method resets the counters.
    pub fn reset_server_ctl_counts(&self) {
        self.server_state_mut().conf_reload_count = 0;
    }

    pub async fn token(&self, user: &str) -> AuthToken {
        let jid = name_to_jid(self, user).await.unwrap();
        self.mock_auth_service()
            .state()
            .sessions
            .iter()
            .find_map(|(k, v)| if v.jid == jid { Some(k) } else { None })
            .expect(&user_missing(user))
            .clone()
    }

    pub fn scenario_workspace_invitation(&self) -> (EmailAddress, Invitation) {
        self.scenario_workspace_invitation
            .as_ref()
            .expect("Current scenario invitation not stored by previous steps")
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
        // FIX: Since we open two different connections for reading
        //   and writing, we need in-memory databases to share their cache
        //.  (otherwise the read-only connection could never see anything).
        //   This creates a named in-memory database (see
        //   [“In-memory Databases And Shared Cache” in SQLite’s “In-Memory Databases” documentation](https://sqlite.org/inmemorydb.html#sharedmemdb)).
        //   We also have to use a unique name because all tests are spawned
        //   in the same process, which means using a constant file name
        //   (e.g. `sqlite:file:test?mode=memory&cache=shared`) would result in
        //   “UNIQUE constraint failed” errors from the database every 2nd test.
        let filename = random_string_alphanumeric(16);
        let db_url = format!("sqlite:file:{filename}?mode=memory&cache=shared");
        std::env::set_var("PROSE_API__DATABASES__MAIN__URL", db_url);

        let db = {
            let config = AppConfig::from_path(CONFIG_PATH.as_path())
                .expect(&format!("Invalid config file at {}", CONFIG_PATH.display()));

            DatabaseRwConnectionPools {
                read: db_conn(&config.api.databases.main_read).await,
                write: db_conn(&config.api.databases.main_write).await,
            }
        };

        // NOTE: We need to run migrations here before they run
        //   in the API because we need to perform actions on
        //   the database before the API starts (it’s not started
        //   by default, since we also test the behavior at startup).
        if let Err(err) = run_migrations(&db.write).await {
            panic!("Could not run migrations in tests: {err}");
        }

        let mock_email_notifier = Arc::new(MockNotifier::<EmailNotification>::default());
        let mock_network_checker = Arc::new(MockNetworkChecker::default());
        let mock_pod_version_service = Arc::new(MockPodVersionService::default());
        let mock_identity_provider = Arc::new(MockIdentityProvider {
            implem: VcardIdentityProvider,
        });

        let mut world = Self {
            app_config: None,
            db,
            api: None,
            config_overrides: Figment::new(),
            result: None,
            scenario_workspace_invitation: None,
            xmpp_service: None,
            mock_xmpp_service: None,
            workspace_service: None,
            mock_workspace_service: None,
            auth_service: None,
            mock_auth_service: None,
            licensing_service: None,
            #[cfg(feature = "test")]
            mock_licensing_service: None,
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
            server_service: None,
            mock_server_service: None,
            identity_provider: IdentityProvider::new(mock_identity_provider.clone()),
            mock_identity_provider,
            user_repository: None,
            mock_user_repository: None,
            invitation_repository: None,
            mock_invitation_repository: None,
            user_application_service: None,
            mock_user_application_service: None,
            mock_invitation_application_service: None,

            mock_server_state: Default::default(),
            mock_auth_service_state: Default::default(),
            mock_xmpp_service_state: Default::default(),
            mock_workspace_service_state: None,
        };

        reload_config(&mut world);

        world
    }
}

// MARK: - Boilerplate

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

    pub fn app_config(&self) -> Arc<AppConfig> {
        self.app_config
            .as_ref()
            .expect("app_config not initialized")
            .clone()
    }
    pub fn mock_licensing_service(&self) -> &MockLicensingService {
        self.mock_licensing_service
            .as_ref()
            .expect("mock_licensing_service not initialized")
    }
    pub fn licensing_service(&self) -> &LicensingService {
        self.licensing_service
            .as_ref()
            .expect("licensing_service not initialized")
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
    pub fn user_repository(&self) -> &UserRepository {
        self.user_repository
            .as_ref()
            .expect("user_repository not initialized")
    }
    pub fn auth_service(&self) -> &AuthService {
        self.auth_service
            .as_ref()
            .expect("auth_service not initialized")
    }
    pub fn mock_auth_service(&self) -> &MockAuthService {
        self.mock_auth_service
            .as_ref()
            .expect("mock_auth_service not initialized")
    }
    pub fn mock_user_repository(&self) -> &Arc<MockUserRepository> {
        self.mock_user_repository
            .as_ref()
            .expect("mock_user_repository not initialized")
    }
    pub fn mock_invitation_repository(&self) -> &Arc<MockInvitationRepository> {
        self.mock_invitation_repository
            .as_ref()
            .expect("mock_invitation_repository not initialized")
    }
    pub fn invitation_repository(&self) -> &InvitationRepository {
        self.invitation_repository
            .as_ref()
            .expect("invitation_repository not initialized")
    }
    pub fn user_application_service(&self) -> &UserApplicationService {
        self.user_application_service
            .as_ref()
            .expect("user_application_service not initialized")
    }
    #[allow(unused)]
    pub fn mock_user_application_service(&self) -> &Arc<MockUserService> {
        self.mock_user_application_service
            .as_ref()
            .expect("mock_user_application_service not initialized")
    }
    pub fn server_service(&self) -> &ProsePodServerService {
        self.server_service
            .as_ref()
            .expect("server_service not initialized")
    }
    pub fn mock_server_service(&self) -> &Arc<MockServerService> {
        self.mock_server_service
            .as_ref()
            .expect("mock_server_service not initialized")
    }
    pub fn xmpp_service(&self) -> &XmppService {
        self.xmpp_service
            .as_ref()
            .expect("xmpp_service not initialized")
    }
    pub fn mock_xmpp_service(&self) -> &Arc<MockXmppService> {
        self.mock_xmpp_service
            .as_ref()
            .expect("mock_xmpp_service not initialized")
    }
    pub fn workspace_service(&self) -> &WorkspaceService {
        self.workspace_service
            .as_ref()
            .expect("workspace_service not initialized")
    }
    #[allow(unused)]
    pub fn mock_workspace_service(&self) -> &Arc<MockWorkspaceService> {
        self.mock_workspace_service
            .as_ref()
            .expect("mock_workspace_service not initialized")
    }
    pub fn mock_workspace_service_state(&mut self) -> &Arc<RwLock<MockWorkspaceServiceState>> {
        let server_domain = self.app_config().server_domain().to_string();
        &*self.mock_workspace_service_state.get_or_insert_with(|| {
            Arc::new(RwLock::new(MockWorkspaceServiceState {
                workspace: Workspace {
                    name: server_domain,
                    icon: None,
                    accent_color: None,
                },
            }))
        })
    }

    pub fn server_config_manager(&self) -> ServerConfigManager {
        ServerConfigManager::new(
            self.db.clone(),
            self.app_config().clone(),
            self.server_service().clone(),
        )
    }

    #[allow(unused)]
    pub fn member_service(&self) -> MemberService {
        MemberService::new(
            self.user_repository().clone(),
            self.user_application_service().clone(),
            self.app_config().server_domain().to_owned(),
            self.xmpp_service().clone(),
            self.auth_service().clone(),
            None,
            &self.app_config().api.member_enriching,
        )
    }

    pub fn invitation_application_service(&self) -> InvitationApplicationService {
        InvitationApplicationService {
            implem: Arc::new(MockInvitationService {
                server: self.mock_server_service().clone(),
                mock_invitation_repository: self.mock_invitation_repository().clone(),
                mock_user_repository: self.mock_user_repository().clone(),
                mock_auth_service: self.mock_auth_service().clone(),
            }),
        }
    }

    pub fn invitation_service(&self) -> InvitationService {
        InvitationService {
            db: self.db.clone(),
            notification_service: self.notifcation_service(),
            invitation_repository: self.invitation_repository().clone(),
            workspace_service: self.workspace_service().clone(),
            auth_service: self.auth_service().clone(),
            xmpp_service: self.xmpp_service().clone(),
            user_repository: self.user_repository().clone(),
            app_config: self.app_config().clone(),
            invitation_application_service: self.invitation_application_service().clone(),
            licensing_service: self.licensing_service().clone(),
        }
    }

    pub fn notifcation_service(&self) -> NotificationService {
        NotificationService::new(self.email_notifier.clone())
    }

    pub async fn server_config(&self) -> anyhow::Result<ServerConfig> {
        use service::server_config;
        let ref dynamic_server_config = server_config::get(&self.db.read).await?;
        let server_config =
            ServerConfig::with_default_values(dynamic_server_config, &self.app_config());
        Ok(server_config)
    }

    pub fn server_state(&self) -> RwLockReadGuard<'_, MockServerServiceState> {
        self.mock_server_service().state.read().unwrap()
    }

    pub fn server_state_mut<'a>(&'a self) -> RwLockWriteGuard<'a, MockServerServiceState> {
        self.mock_server_service().state.write().unwrap()
    }

    pub fn xmpp_service_state_mut<'a>(&'a self) -> RwLockWriteGuard<'a, MockXmppServiceState> {
        self.mock_xmpp_service().state.write().unwrap()
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
}
