// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::model::{JidNode, MemberRole};
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait as _};
use secrecy::SecretString;

use crate::{
    config::AppConfig,
    model::{Member, ServerConfig, Workspace},
    repositories::{
        MemberRepository, ServerConfigCreateForm, WorkspaceCreateForm, WorkspaceRepository,
    },
    services::{
        server_ctl::ServerCtl,
        server_manager::{self, ServerManager},
        user_service::{UserCreateError, UserService},
    },
    util::bare_jid_from_username,
};

pub enum InitController {}

impl InitController {
    pub async fn init_server_config(
        db: &DatabaseConnection,
        server_ctl: &ServerCtl,
        app_config: &AppConfig,
        server_config: impl Into<ServerConfigCreateForm>,
    ) -> Result<ServerConfig, InitServerConfigError> {
        let server_config =
            ServerManager::init_server_config(db, server_ctl, app_config, server_config)
                .await
                .map_err(InitServerConfigError::CouldNotInitServerConfig)?;

        Ok(server_config)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitServerConfigError {
    #[error("Could not init server config: {0}")]
    CouldNotInitServerConfig(server_manager::Error),
}

impl InitController {
    pub async fn init_workspace(
        db: &DatabaseConnection,
        form: impl Into<WorkspaceCreateForm>,
    ) -> Result<Workspace, InitWorkspaceError> {
        // Check that the workspace isn't already initialized.
        let None = WorkspaceRepository::get(db).await? else {
            return Err(InitWorkspaceError::WorkspaceAlreadyInitialized);
        };

        let workspace = WorkspaceRepository::create(db, form).await?;

        Ok(workspace)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitWorkspaceError {
    #[error("Workspace already initialized.")]
    WorkspaceAlreadyInitialized,
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
impl InitController {
    pub async fn init_first_account(
        db: &DatabaseConnection,
        server_config: &ServerConfig,
        user_service: &UserService<'_>,
        form: impl Into<InitFirstAccountForm>,
    ) -> Result<Member, InitFirstAccountError> {
        let form = form.into();
        let jid = bare_jid_from_username(&form.username, &server_config)
            .map_err(InitFirstAccountError::InvalidJid)?;

        if MemberRepository::count(db).await? > 0 {
            return Err(InitFirstAccountError::FirstAccountAlreadyCreated);
        }

        let txn = db.begin().await?;
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
