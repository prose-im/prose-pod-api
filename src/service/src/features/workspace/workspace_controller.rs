// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use prose_xmpp::stanza::VCard4;
use tracing::info;
use validator::Validate;

use crate::{
    models::{Avatar, AvatarOwned, Color},
    util::either::Either,
    xmpp::VCard,
};

use super::{errors::WorkspaceAlreadyInitialized, GetWorkspaceError, Workspace, WorkspaceService};

// MARK: Init workspace

pub async fn init_workspace(
    workspace_service: &WorkspaceService,
    form: impl Into<Workspace>,
) -> Result<Workspace, Either<WorkspaceAlreadyInitialized, anyhow::Error>> {
    // Check that the workspace isn't already initialized.
    if workspace_service.get_workspace_name().await.is_ok() {
        return Err(Either::E1(WorkspaceAlreadyInitialized));
    };

    let workspace: Workspace = form.into();

    workspace_service
        .set_workspace_vcard(&VCard4::from(&workspace))
        .await
        .context("Could not set workspace vCard")?;

    info!("Workspace initialized successfully.");

    Ok(workspace)
}

pub async fn is_workspace_initialized(
    workspace_service: &WorkspaceService,
) -> anyhow::Result<bool> {
    workspace_service.is_workspace_initialized().await
}

// MARK: Get one

pub async fn get_workspace(
    workspace_service: &WorkspaceService,
) -> Result<Workspace, GetWorkspaceError> {
    workspace_service.get_workspace().await
}

// MARK: Get/set fields

pub async fn get_workspace_accent_color(
    workspace_service: &WorkspaceService,
) -> Result<Option<Color>, GetWorkspaceError> {
    workspace_service.get_workspace_accent_color().await
}
pub async fn set_workspace_accent_color(
    workspace_service: &WorkspaceService,
    accent_color: Option<Color>,
) -> Result<Option<Color>, GetWorkspaceError> {
    workspace_service
        .set_workspace_accent_color(accent_color)
        .await
}

pub async fn get_workspace_name(
    workspace_service: &WorkspaceService,
) -> Result<String, GetWorkspaceError> {
    workspace_service.get_workspace_name().await
}
pub async fn set_workspace_name(
    workspace_service: &WorkspaceService,
    name: String,
) -> Result<String, GetWorkspaceError> {
    workspace_service.set_workspace_name(name).await
}

pub async fn get_workspace_icon<'a>(
    workspace_service: &'a WorkspaceService,
) -> anyhow::Result<Option<AvatarOwned>, anyhow::Error> {
    workspace_service.get_workspace_icon().await
}
pub async fn set_workspace_icon<'a>(
    workspace_service: &WorkspaceService,
    icon: Avatar<'a>,
) -> anyhow::Result<()> {
    workspace_service.set_workspace_icon(icon).await
}

// MARK: Patch one

#[derive(Debug, Clone)]
#[derive(Validate, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
pub struct PatchWorkspaceDetailsRequest {
    #[validate(length(min = 1, max = 48), non_control_character)]
    pub name: Option<String>,

    #[serde(
        default,
        deserialize_with = "crate::util::deserialize_null_as_some_none"
    )]
    #[validate(skip)]
    pub accent_color: Option<Option<Color>>,
}

pub async fn patch_workspace(
    workspace_service: &WorkspaceService,
    req: PatchWorkspaceDetailsRequest,
) -> Result<Workspace, GetWorkspaceError> {
    let mut workspace = workspace_service.get_workspace().await?;

    if let Some(name) = req.name {
        workspace.name = name
    }
    if let Some(accent_color) = req.accent_color {
        workspace.accent_color = accent_color
    }

    let vcard = VCard::from(&workspace);
    workspace_service.set_workspace_vcard(&vcard).await?;

    Ok(workspace)
}
