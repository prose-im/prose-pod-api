// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Context as _;
use jid::DomainRef;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait as _};
use secrecy::SecretString;
use tracing::{info, instrument};

use crate::{
    members::{
        Member, MemberRepository, MemberRole, UnauthenticatedMemberService, UserCreateError,
    },
    models::JidNode,
    secrets::SecretsStore,
    util::bare_jid_from_username,
    workspace::{Workspace, WorkspaceService, WorkspaceServiceInitError},
    xmpp::XmppServiceInner,
    AppConfig,
};

pub struct InitService {
    pub db: Arc<DatabaseConnection>,
}

impl InitService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn init_workspace(
        &self,
        app_config: Arc<AppConfig>,
        secrets_store: Arc<SecretsStore>,
        xmpp_service: Arc<XmppServiceInner>,
        server_domain: &DomainRef,
        form: impl Into<Workspace>,
    ) -> Result<Workspace, InitWorkspaceError> {
        let workspace = form.into();

        let workspace_service =
            WorkspaceService::new(xmpp_service, app_config, server_domain, secrets_store)?;

        // Check that the workspace isn't already initialized.
        let None = workspace_service.get_workspace_name().await.ok() else {
            return Err(InitWorkspaceError::WorkspaceAlreadyInitialized);
        };

        workspace_service
            .set_workspace_vcard(&workspace.clone().into())
            .await
            .context("Could not set workspace vCard")?;

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
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl InitService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn init_first_account(
        &self,
        server_domain: &DomainRef,
        member_service: &UnauthenticatedMemberService,
        form: impl Into<InitFirstAccountForm>,
    ) -> Result<Member, InitFirstAccountError> {
        let form = form.into();
        let jid = bare_jid_from_username(&form.username, server_domain)
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

// MARK: BOILERPLATE

impl From<WorkspaceServiceInitError> for InitWorkspaceError {
    fn from(value: WorkspaceServiceInitError) -> Self {
        match value {
            WorkspaceServiceInitError::WorkspaceXmppAccountNotInitialized => {
                Self::XmppAccountNotInitialized
            }
        }
    }
}
