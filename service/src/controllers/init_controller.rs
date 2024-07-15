// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sea_orm::{DatabaseConnection, DbErr, NotSet, Set, TransactionTrait as _};
use secrecy::SecretString;
use tracing::debug;

use crate::{
    config::AppConfig,
    entity::workspace,
    model::{
        JidNode, Member, MemberRole, ServerConfig, ServiceSecrets, ServiceSecretsStore, Workspace,
    },
    repositories::{MemberRepository, ServerConfigCreateForm, WorkspaceRepository},
    services::{
        auth_service::{self, AuthService},
        jwt_service::{InvalidJwtClaimError, JWTError},
        server_ctl::{self, ServerCtl},
        server_manager::{self, ServerManager},
        user_service::{UserCreateError, UserService},
        xmpp_service::XmppServiceInner,
    },
    util::bare_jid_from_username,
};

use super::workspace_controller::{self, WorkspaceController, WorkspaceControllerInitError};

pub struct InitController<'r> {
    pub db: &'r DatabaseConnection,
}

impl<'r> InitController<'r> {
    pub async fn init_server_config(
        &self,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        auth_service: &AuthService,
        secrets_store: &ServiceSecretsStore,
        server_config: impl Into<ServerConfigCreateForm>,
    ) -> Result<ServerConfig, InitServerConfigError> {
        let server_config =
            ServerManager::init_server_config(self.db, server_ctl, app_config, server_config)
                .await
                .map_err(InitServerConfigError::CouldNotInitServerConfig)?;

        // Create workspace XMPP account
        Self::create_service_account(
            app_config.workspace_jid(),
            server_ctl,
            auth_service,
            secrets_store,
        )
        .await?;

        Ok(server_config)
    }

    async fn create_service_account(
        jid: BareJid,
        server_ctl: &ServerCtl,
        auth_service: &AuthService,
        secrets_store: &ServiceSecretsStore,
    ) -> Result<(), CreateServiceAccountError> {
        debug!("Creating service account '{jid}'…");

        // Generate a very strong random password
        // NOTE: Code taken from <https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters>.
        let password: SecretString = thread_rng()
            .sample_iter(&Alphanumeric)
            // 256 characters because why not
            .take(256)
            .map(char::from)
            .collect::<String>()
            .into();

        // Create the XMPP user account
        server_ctl.add_user(&jid, &password).await?;

        // Log in as the service account (to get a JWT with access tokens)
        let jwt = auth_service.log_in(&jid, &password).await?;
        let jwt = auth_service.verify(&jwt)?;

        // Read the access tokens from the JWT
        let prosody_token = jwt
            .prosody_token()
            .map_err(CreateServiceAccountError::MissingProsodyToken)?;

        // Store the secrets
        let secrets = ServiceSecrets { prosody_token };
        secrets_store.set_secrets(jid, secrets);

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitServerConfigError {
    #[error("Could not init server config: {0}")]
    CouldNotInitServerConfig(server_manager::Error),
    #[error("Could not create service XMPP account: {0}")]
    CouldNotCreateServiceAccount(#[from] CreateServiceAccountError),
}

#[derive(Debug, thiserror::Error)]
pub enum CreateServiceAccountError {
    #[error("Could not create XMPP account: {0}")]
    CouldNotCreateXmppAccount(#[from] server_ctl::Error),
    #[error("Could not log in: {0}")]
    CouldNotLogIn(#[from] auth_service::Error),
    #[error("The just-created JWT is invalid: {0}")]
    InvalidJwt(#[from] JWTError),
    #[error("The just-created JWT doesn't contain a Prosody token: {0}")]
    MissingProsodyToken(InvalidJwtClaimError),
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

impl<'r> InitController<'r> {
    pub async fn init_workspace(
        &self,
        app_config: &AppConfig,
        secrets_store: &ServiceSecretsStore,
        xmpp_service: &XmppServiceInner,
        form: impl Into<WorkspaceCreateForm>,
    ) -> Result<Workspace, InitWorkspaceError> {
        // Check that the workspace isn't already initialized.
        let None = WorkspaceRepository::get(self.db).await? else {
            return Err(InitWorkspaceError::WorkspaceAlreadyInitialized);
        };

        let form = form.into();

        let workspace = WorkspaceRepository::create(self.db, form.clone()).await?;

        let workspace_controller =
            WorkspaceController::new(self.db, xmpp_service, app_config, secrets_store)?;
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

impl<'r> InitController<'r> {
    pub async fn init_first_account(
        &self,
        server_config: &ServerConfig,
        user_service: &UserService<'_>,
        form: impl Into<InitFirstAccountForm>,
    ) -> Result<Member, InitFirstAccountError> {
        let form = form.into();
        let jid = bare_jid_from_username(&form.username, &server_config)
            .map_err(InitFirstAccountError::InvalidJid)?;

        if MemberRepository::count(self.db).await? > 0 {
            return Err(InitFirstAccountError::FirstAccountAlreadyCreated);
        }

        let txn = self.db.begin().await?;
        let member = user_service
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
