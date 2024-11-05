// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{DatabaseConnection, DbErr, IntoActiveModel as _};

use crate::{
    features::{
        secrets::SecretsStore,
        server_config::ServerConfig,
        workspace::{Workspace, WorkspaceRepository},
        xmpp::{xmpp_service, AvatarData, XmppService, XmppServiceContext, XmppServiceInner},
    },
    sea_orm::{ActiveModelTrait as _, Set},
    AppConfig,
};

pub struct WorkspaceController<'r> {
    db: &'r DatabaseConnection,
    xmpp_service: XmppService<'r>,
}

impl<'r> WorkspaceController<'r> {
    pub fn new(
        db: &'r DatabaseConnection,
        xmpp_service: &'r XmppServiceInner,
        app_config: &'r AppConfig,
        server_config: &ServerConfig,
        secrets_store: &'r SecretsStore,
    ) -> Result<Self, WorkspaceControllerInitError> {
        let workspace_jid = app_config.workspace_jid(&server_config.domain);
        let prosody_token = secrets_store
            .get_service_account_prosody_token(&workspace_jid)
            .ok_or(WorkspaceControllerInitError::WorkspaceXmppAccountNotInitialized)?;
        let ctx = XmppServiceContext {
            bare_jid: workspace_jid,
            prosody_token,
        };
        let xmpp_service = XmppService::new(xmpp_service, ctx);
        Ok(Self { db, xmpp_service })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceControllerInitError {
    #[error("Workspace XMPP account not initialized.")]
    WorkspaceXmppAccountNotInitialized,
}

impl<'r> WorkspaceController<'r> {
    pub async fn get_workspace(&self) -> Result<Workspace, Error> {
        WorkspaceRepository::get(self.db)
            .await?
            .ok_or(Error::WorkspaceNotInitialized)
    }

    pub async fn get_workspace_name(&self) -> Result<String, Error> {
        // FIXME: Get `FN` instead of `NICK`
        let nickname = self
            .xmpp_service
            .get_own_nickname()
            .await?
            .ok_or(Error::WorkspaceNotInitialized)?;
        Ok(nickname)
    }

    pub async fn set_workspace_name(&self, name: String) -> Result<String, Error> {
        // FIXME: Set `FN` instead of `NICK`
        self.xmpp_service.set_own_nickname(&name).await?;
        Ok(name)
    }

    pub async fn get_workspace_icon(&self) -> Result<Option<AvatarData>, Error> {
        let avatar = self.xmpp_service.get_own_avatar().await?;
        Ok(avatar)
    }
    pub async fn set_workspace_icon(&self, png_data: Vec<u8>) -> Result<(), Error> {
        self.xmpp_service.set_own_avatar(png_data).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GetWorkspaceAccentColorResponse {
    pub color: Option<String>,
}

impl<'r> WorkspaceController<'r> {
    pub async fn get_workspace_accent_color(&self) -> Result<Option<String>, Error> {
        Ok(self.get_workspace().await?.accent_color)
    }
}

impl<'r> WorkspaceController<'r> {
    pub async fn set_workspace_accent_color(&self, color: String) -> Result<Option<String>, Error> {
        let workspace = self.get_workspace().await?;

        let mut active = workspace.into_active_model();
        // TODO: Validate `color`
        active.accent_color = Set(Some(color));
        let workspace = active.update(self.db).await?;

        Ok(workspace.accent_color)
    }
}

pub type Error = WorkspaceControllerError;

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceControllerError {
    #[error("Workspace not initialized.")]
    WorkspaceNotInitialized,
    #[error("XmppServiceError: {0}")]
    XmppServiceError(#[from] xmpp_service::Error),
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
