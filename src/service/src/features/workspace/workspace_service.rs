// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context;
use prose_xmpp::stanza::VCard4;
use tracing::instrument;

use crate::{
    models::{Avatar, AvatarOwned, Color},
    workspace::Workspace,
    xmpp::{XmppService, XmppServiceContext},
};

use super::errors::WorkspaceNotInitialized;

#[derive(Debug, Clone)]
pub struct WorkspaceService {
    pub xmpp_service: XmppService,
    pub ctx: XmppServiceContext,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceServiceInitError {
    #[error("Workspace XMPP account not initialized.")]
    WorkspaceXmppAccountNotInitialized,
}

impl WorkspaceService {
    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn is_workspace_initialized(&self) -> anyhow::Result<bool> {
        let vcard =
            (self.xmpp_service.get_own_vcard(&self.ctx).await).context("XmppService error")?;
        Ok(vcard.is_some())
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace(&self) -> Result<Workspace, GetWorkspaceError> {
        let vcard = self.get_workspace_vcard().await?;
        let mut workspace = Workspace::try_from(vcard)?;
        // Avatars are not stored in vCards.
        // TODO: Avoid this copy?
        workspace.icon = self.get_workspace_icon().await?;
        Ok(workspace)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_vcard(&self) -> Result<VCard4, GetWorkspaceError> {
        let vcard = (self.xmpp_service.get_own_vcard(&self.ctx).await)
            .context("XmppService error")?
            .ok_or(WorkspaceNotInitialized::WithReason("No vCard."))?;
        Ok(vcard)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_vcard(&self, vcard: &VCard4) -> anyhow::Result<()> {
        (self.xmpp_service.set_own_vcard(&self.ctx, vcard).await).context("XmppService error")?;
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
        self.set_workspace_vcard(&VCard4::from(&workspace)).await?;
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
        self.set_workspace_vcard(&VCard4::from(&workspace)).await?;
        Ok(accent_color)
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn get_workspace_icon(&self) -> anyhow::Result<Option<AvatarOwned>> {
        (self.xmpp_service.get_own_avatar(&self.ctx).await).context("XmppService error")
    }

    #[instrument(level = "trace", skip_all, err(level = "trace"))]
    pub async fn set_workspace_icon<'a>(&self, icon: Avatar<'a>) -> anyhow::Result<()> {
        (self.xmpp_service.set_own_avatar(&self.ctx, icon).await).context("XmppService error")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetWorkspaceError {
    #[error("{0}")]
    WorkspaceNotInitialized(#[from] WorkspaceNotInitialized),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
