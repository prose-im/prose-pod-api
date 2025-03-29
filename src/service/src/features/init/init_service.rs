// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use sea_orm::{DatabaseConnection, DbErr, TransactionTrait as _};
use secrecy::SecretString;
use tracing::{info, instrument};

use crate::{
    auth::{auth_service, AuthService},
    members::{
        Member, MemberRepository, MemberRole, UnauthenticatedMemberService, UserCreateError,
    },
    models::JidNode,
    secrets::SecretsStore,
    server_config::{ServerConfig, ServerConfigCreateForm},
    util::bare_jid_from_username,
    workspace::{workspace_service, Workspace, WorkspaceService, WorkspaceServiceInitError},
    xmpp::{server_manager, CreateServiceAccountError, ServerCtl, ServerManager, XmppServiceInner},
    AppConfig,
};

pub struct InitService {
    pub db: Arc<DatabaseConnection>,
}

impl InitService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn init_server_config(
        &self,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        auth_service: &AuthService,
        secrets_store: &SecretsStore,
        server_config: impl Into<ServerConfigCreateForm>,
    ) -> Result<ServerConfig, InitServerConfigError> {
        // Initialize XMPP server configuration
        let server_config =
            ServerManager::init_server_config(&self.db, server_ctl, app_config, server_config)
                .await
                .map_err(InitServerConfigError::CouldNotInitServerConfig)?;

        // Register OAuth 2.0 client
        auth_service
            .register_oauth2_client()
            .await
            .map_err(InitServerConfigError::CouldNotRegisterOAuth2Client)?;

        // Create service XMPP accounts
        ServerManager::create_service_accounts(
            &server_config.domain,
            server_ctl,
            app_config,
            auth_service,
            secrets_store,
        )
        .await?;

        Ok(server_config)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitServerConfigError {
    #[error("Could not init server config: {0}")]
    CouldNotInitServerConfig(server_manager::Error),
    #[error("Could register OAuth 2.0 client: {0}")]
    CouldNotRegisterOAuth2Client(auth_service::Error),
    #[error("Could not create service XMPP account: {0}")]
    CouldNotCreateServiceAccount(#[from] CreateServiceAccountError),
}

impl InitService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn init_workspace(
        &self,
        app_config: Arc<AppConfig>,
        secrets_store: Arc<SecretsStore>,
        xmpp_service: Arc<XmppServiceInner>,
        server_config: &ServerConfig,
        form: impl Into<Workspace>,
    ) -> Result<Workspace, InitWorkspaceError> {
        let workspace = form.into();

        let workspace_service =
            WorkspaceService::new(xmpp_service, app_config, server_config, secrets_store)?;

        // Check that the workspace isn't already initialized.
        let None = workspace_service.get_workspace_name().await.ok() else {
            return Err(InitWorkspaceError::WorkspaceAlreadyInitialized);
        };

        workspace_service
            .set_workspace_vcard(&workspace.clone().into())
            .await
            .map_err(InitWorkspaceError::CouldNotSetWorkspaceVCard)?;

        info!("Workspace initialized successfully.");

        Ok(workspace)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitWorkspaceError {
    #[error("Workspace already initialized.")]
    WorkspaceAlreadyInitialized,
    #[error("Workspace XMPP account not initialized.")]
    XmppAccountNotInitialized,
    #[error("Could not set workspace vCard: {0}")]
    CouldNotSetWorkspaceVCard(workspace_service::Error),
}

impl From<WorkspaceServiceInitError> for InitWorkspaceError {
    fn from(value: WorkspaceServiceInitError) -> Self {
        match value {
            WorkspaceServiceInitError::WorkspaceXmppAccountNotInitialized => {
                Self::XmppAccountNotInitialized
            }
        }
    }
}

impl InitService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn init_first_account(
        &self,
        server_config: &ServerConfig,
        member_service: &UnauthenticatedMemberService,
        form: impl Into<InitFirstAccountForm>,
    ) -> Result<Member, InitFirstAccountError> {
        let form = form.into();
        let jid = bare_jid_from_username(&form.username, &server_config)
            .map_err(InitFirstAccountError::InvalidJid)?;

        if MemberRepository::count(self.db.as_ref()).await? > 0 {
            return Err(InitFirstAccountError::FirstAccountAlreadyCreated);
        }

        let txn = self.db.begin().await?;
        let member = member_service
            .create_user(
                &txn,
                &jid,
                &form.password,
                &form.nickname,
                &Some(MemberRole::Admin),
            )
            .await
            .map_err(InitFirstAccountError::CouldNotCreateFirstAccount)?;
        txn.commit().await?;

        Ok(member)
    }
}

#[derive(Debug)]
pub struct InitFirstAccountForm {
    pub username: JidNode,
    pub password: SecretString,
    pub nickname: String,
}

#[derive(Debug, thiserror::Error)]
pub enum InitFirstAccountError {
    #[error("First account already created.")]
    FirstAccountAlreadyCreated,
    #[error("Invalid JID: {0}")]
    InvalidJid(String),
    #[error("Could not create first account: {0}")]
    CouldNotCreateFirstAccount(UserCreateError),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
