// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_xmpp::stanza::VCard4;
use tracing::instrument;

use crate::{
    secrets::SecretsStore,
    server_config::ServerConfig,
    workspace::Workspace,
    xmpp::{xmpp_service, AvatarData, XmppService, XmppServiceContext, XmppServiceInner},
    AppConfig,
};

use super::models;

#[derive(Clone)]
pub struct WorkspaceService {
    xmpp_service: XmppService,
}

impl WorkspaceService {
    pub fn new(
        xmpp_service: Arc<XmppServiceInner>,
        app_config: Arc<AppConfig>,
        server_config: &ServerConfig,
        secrets_store: Arc<SecretsStore>,
    ) -> Result<Self, WorkspaceServiceInitError> {
        let workspace_jid = app_config.workspace_jid(&server_config.domain);
        let prosody_token = secrets_store
            .get_service_account_prosody_token(&workspace_jid)
            .ok_or(WorkspaceServiceInitError::WorkspaceXmppAccountNotInitialized)?;
        let ctx = XmppServiceContext {
            bare_jid: workspace_jid,
            prosody_token,
        };
        let xmpp_service = XmppService::new(xmpp_service, ctx);
        Ok(Self { xmpp_service })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceServiceInitError {
    #[error("Workspace XMPP account not initialized.")]
    WorkspaceXmppAccountNotInitialized,
}

impl WorkspaceService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace(&self) -> Result<Workspace, Error> {
        let vcard = self.get_workspace_vcard().await?;
        let mut workspace = Workspace::try_from(vcard)?;
        // Avatars are not stored in vCards.
        workspace.icon = self.get_workspace_icon_base64().await?;
        Ok(workspace)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_vcard(&self) -> Result<VCard4, Error> {
        let vcard = self
            .xmpp_service
            .get_own_vcard()
            .await?
            .ok_or(Error::WorkspaceNotInitialized("No vCard."))?;
        Ok(vcard)
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_vcard(&self, vcard: &VCard4) -> Result<(), Error> {
        self.xmpp_service.set_own_vcard(vcard).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_name(&self) -> Result<String, Error> {
        let vcard = self.get_workspace_vcard().await?;
        let workspace = Workspace::try_from(vcard)?;
        Ok(workspace.name)
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_name(&self, name: String) -> Result<String, Error> {
        let mut workspace = self.get_workspace().await?;
        workspace.name = name.clone();
        self.set_workspace_vcard(&workspace.into()).await?;
        Ok(name)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_accent_color(&self) -> Result<Option<String>, Error> {
        let vcard = self.get_workspace_vcard().await?;
        let workspace = Workspace::try_from(vcard)?;
        Ok(workspace.accent_color)
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_accent_color(&self, accent_color: String) -> Result<String, Error> {
        let mut workspace = self.get_workspace().await?;
        workspace.accent_color = Some(accent_color.clone());
        self.set_workspace_vcard(&workspace.into()).await?;
        Ok(accent_color)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_icon(&self) -> Result<Option<AvatarData>, Error> {
        let avatar = self.xmpp_service.get_own_avatar().await?;
        Ok(avatar)
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_icon_base64(&self) -> Result<Option<String>, Error> {
        let avatar_data = self.get_workspace_icon().await?;
        Ok(avatar_data.map(|d| d.base64().into_owned()))
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_icon(&self, png_data: Vec<u8>) -> Result<(), Error> {
        self.xmpp_service.set_own_avatar(png_data).await?;
        Ok(())
    }
}

pub type Error = WorkspaceServiceError;

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceServiceError {
    #[error("Workspace not initialized: {0}")]
    WorkspaceNotInitialized(&'static str),
    #[error("XmppServiceError: {0}")]
    XmppServiceError(#[from] xmpp_service::Error),
}

impl From<models::WorkspaceNameNotInitialized> for WorkspaceServiceError {
    fn from(_: models::WorkspaceNameNotInitialized) -> Self {
        Self::WorkspaceNotInitialized("Missing name.")
    }
}
