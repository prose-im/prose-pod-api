// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use base64::{engine::general_purpose, Engine as _};
use mime::Mime;
use tracing::info;

use crate::{
    util::{detect_image_mime_type, either::Either},
    xmpp::{xmpp_service::Avatar, VCard},
};

use super::{errors::WorkspaceAlreadyInitialized, GetWorkspaceError, Workspace, WorkspaceService};

// MARK: INIT WORKSPACE

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
        .set_workspace_vcard(&workspace.clone().into())
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

// MARK: GET ONE

pub async fn get_workspace(
    workspace_service: &WorkspaceService,
) -> Result<Workspace, GetWorkspaceError> {
    workspace_service.get_workspace().await
}

// MARK: GET/SET FIELDS

pub async fn get_workspace_accent_color(
    workspace_service: &WorkspaceService,
) -> Result<Option<String>, GetWorkspaceError> {
    workspace_service.get_workspace_accent_color().await
}
pub async fn set_workspace_accent_color(
    workspace_service: &WorkspaceService,
    accent_color: Option<String>,
) -> Result<Option<String>, GetWorkspaceError> {
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

pub async fn get_workspace_icon(
    workspace_service: &WorkspaceService,
) -> anyhow::Result<Option<Avatar>, anyhow::Error> {
    workspace_service.get_workspace_icon().await
}
pub async fn set_workspace_icon(
    workspace_service: &WorkspaceService,
    mime: Option<Mime>,
    base64: String,
) -> Result<Avatar, SetWorkspaceIconError> {
    let mime = (detect_image_mime_type(&base64, mime))
        .ok_or(SetWorkspaceIconError::UnsupportedMediaType)?;

    let icon_data = general_purpose::STANDARD.decode(&base64)?;

    (workspace_service.set_workspace_icon(icon_data, &mime).await)?;

    Ok(Avatar { base64, mime })
}

#[derive(Debug, thiserror::Error)]
pub enum SetWorkspaceIconError {
    #[error("Image data should be Base64-encoded. Error: {0}")]
    BadImageDataFormat(#[from] base64::DecodeError),
    #[error("Unsupported media type. Supported: {}.",
        [
            mime::IMAGE_PNG.to_string(),
            mime::IMAGE_GIF.to_string(),
            mime::IMAGE_JPEG.to_string(),
        ]
        .join(", ")
    )]
    UnsupportedMediaType,
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

// MARK: PATCH ONE

#[derive(Debug, Clone)]
#[derive(serdev::Deserialize)]
pub struct PatchWorkspaceDetailsRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::util::deserialize_some")]
    pub accent_color: Option<Option<String>>,
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

    let vcard = VCard::from(workspace.clone());
    workspace_service.set_workspace_vcard(&vcard).await?;

    Ok(workspace)
}
