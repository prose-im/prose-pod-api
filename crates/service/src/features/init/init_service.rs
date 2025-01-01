// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use sea_orm::{DatabaseConnection, DbErr, NotSet, Set, TransactionTrait as _};
use secrecy::SecretString;

use crate::{
    auth::{auth_service, AuthService},
    members::{Member, MemberRepository, MemberRole, MemberService, UserCreateError},
    models::JidNode,
    secrets::SecretsStore,
    server_config::{ServerConfig, ServerConfigCreateForm},
    util::bare_jid_from_username,
    workspace::{
        entities::workspace, workspace_controller, WorkspaceController,
        WorkspaceControllerInitError, WorkspaceRepository,
    },
    xmpp::{server_manager, CreateServiceAccountError, ServerCtl, ServerManager, XmppServiceInner},
    AppConfig,
};

pub struct InitService {
    pub db: Arc<DatabaseConnection>,
}

impl InitService {
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

#[derive(Debug, Clone)]
pub struct WorkspaceCreateForm {
    pub name: String,
    pub accent_color: Option<Option<String>>,
}

impl Into<workspace::ActiveModel> for WorkspaceCreateForm {
    fn into(self) -> workspace::ActiveModel {
        workspace::ActiveModel {
            accent_color: self.accent_color.map(Set).unwrap_or(NotSet),
            ..Default::default()
        }
    }
}

impl InitService {
    pub async fn init_workspace(
        &self,
        app_config: Arc<AppConfig>,
        secrets_store: Arc<SecretsStore>,
        xmpp_service: Arc<XmppServiceInner>,
        server_config: &ServerConfig,
        form: impl Into<WorkspaceCreateForm>,
    ) -> Result<workspace::Model, InitWorkspaceError> {
        // Check that the workspace isn't already initialized.
        let None = WorkspaceRepository::get(self.db.as_ref()).await? else {
            return Err(InitWorkspaceError::WorkspaceAlreadyInitialized);
        };

        let form = form.into();

        let workspace = WorkspaceRepository::create(self.db.as_ref(), form.clone()).await?;

        let workspace_controller = WorkspaceController::new(
            self.db.clone(),
            xmpp_service,
            app_config,
            server_config,
            secrets_store,
        )?;
        workspace_controller
            .set_workspace_name(form.name)
            .await
            .map_err(InitWorkspaceError::CouldNotSetWorkspaceName)?;

        Ok(workspace)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitWorkspaceError {
    #[error("Workspace already initialized.")]
    WorkspaceAlreadyInitialized,
    #[error("Workspace XMPP account not initialized.")]
    XmppAccountNotInitialized,
    #[error("Could not set workspace name: {0}")]
    CouldNotSetWorkspaceName(workspace_controller::Error),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}

impl From<WorkspaceControllerInitError> for InitWorkspaceError {
    fn from(value: WorkspaceControllerInitError) -> Self {
        match value {
            WorkspaceControllerInitError::WorkspaceXmppAccountNotInitialized => {
                Self::XmppAccountNotInitialized
            }
        }
    }
}

impl InitService {
    pub async fn init_first_account(
        &self,
        server_config: &ServerConfig,
        member_service: &MemberService,
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
