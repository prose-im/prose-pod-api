// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Context;
use jid::BareJid;
use prose_xmpp::stanza::VCard4;
use tracing::instrument;

use crate::{
    models::Color,
    secrets::SecretsStore,
    workspace::Workspace,
    xmpp::{xmpp_service::Avatar, XmppService, XmppServiceContext, XmppServiceInner},
};

use super::errors::WorkspaceNotInitialized;

#[derive(Clone)]
pub struct WorkspaceService {
    xmpp_service: XmppService,
}

impl WorkspaceService {
    pub fn new(
        xmpp_service: XmppServiceInner,
        workspace_jid: BareJid,
        secrets_store: Arc<SecretsStore>,
    ) -> Result<Self, WorkspaceServiceInitError> {
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
    pub async fn is_workspace_initialized(&self) -> anyhow::Result<bool> {
        let vcard = (self.xmpp_service.get_own_vcard().await).context("XmppService error")?;
        Ok(vcard.is_some())
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace(&self) -> Result<Workspace, GetWorkspaceError> {
        let vcard = self.get_workspace_vcard().await?;
        let mut workspace = Workspace::try_from(vcard)?;
        // Avatars are not stored in vCards.
        workspace.icon = self.get_workspace_icon().await?;
        Ok(workspace)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_vcard(&self) -> Result<VCard4, GetWorkspaceError> {
        let vcard = (self.xmpp_service.get_own_vcard().await)
            .context("XmppService error")?
            .ok_or(WorkspaceNotInitialized::WithReason("No vCard."))?;
        Ok(vcard)
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_vcard(&self, vcard: &VCard4) -> anyhow::Result<()> {
        (self.xmpp_service.set_own_vcard(vcard).await).context("XmppService error")?;
        Ok(())
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_name(&self) -> Result<String, GetWorkspaceError> {
        let vcard = self.get_workspace_vcard().await?;
        let workspace = Workspace::try_from(vcard)?;
        Ok(workspace.name)
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_name(&self, name: String) -> Result<String, GetWorkspaceError> {
        let mut workspace = self.get_workspace().await?;
        workspace.name = name.clone();
        self.set_workspace_vcard(&workspace.into()).await?;
        Ok(name)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_accent_color(&self) -> Result<Option<Color>, GetWorkspaceError> {
        let vcard = self.get_workspace_vcard().await?;
        let workspace = Workspace::try_from(vcard)?;
        Ok(workspace.accent_color)
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_accent_color(
        &self,
        accent_color: Option<Color>,
    ) -> Result<Option<Color>, GetWorkspaceError> {
        let mut workspace = self.get_workspace().await?;
        workspace.accent_color = accent_color.clone();
        self.set_workspace_vcard(&workspace.into()).await?;
        Ok(accent_color)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_icon(&self) -> anyhow::Result<Option<Avatar>> {
        (self.xmpp_service.get_own_avatar().await).context("XmppService error")
    }
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_icon(&self, data: Vec<u8>, mime: &mime::Mime) -> anyhow::Result<()> {
        (self.xmpp_service.set_own_avatar(data, mime).await).context("XmppService error")
    }
}

impl WorkspaceService {
    pub async fn migrate_workspace_vcard(&self) -> Result<(), GetWorkspaceError> {
        let mut vcard = self.get_workspace_vcard().await?;

        let workspace = self.get_workspace().await?;
        let expected = VCard4::from(workspace);

        if vcard.kind.is_none() {
            vcard.kind = expected.kind;
        }

        self.set_workspace_vcard(&vcard).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetWorkspaceError {
    #[error("{0}")]
    WorkspaceNotInitialized(#[from] WorkspaceNotInitialized),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
