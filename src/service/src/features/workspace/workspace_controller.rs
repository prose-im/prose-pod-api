// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use validator::Validate;

use crate::{
    auth::AuthToken,
    errors::Forbidden,
    models::{Avatar, Color, DatabaseRwConnectionPools},
    onboarding::is_workspace_initialized,
    prose_pod_server_api::InitWorkspaceRequest,
    util::either::Either,
    workspace::errors::WorkspaceAlreadyInitialized,
};

use super::{Workspace, WorkspaceService};

// MARK: Init workspace

// COMPAT: This route will disappear because the Workspace is now initialized
//   by default, but until then it’ll map to `PATCH /v1/workspace`, with a
//   check that ensures the name is set (not to change behavior).
pub async fn init_workspace(
    db: &DatabaseRwConnectionPools,
    workspace_service: &WorkspaceService,
    req: impl Into<InitWorkspaceRequest>,
) -> Result<Workspace, Either<WorkspaceAlreadyInitialized, anyhow::Error>> {
    workspace_service.init_workspace(&req.into()).await?;

    is_workspace_initialized::set(&db.write, true).await?;

    let workspace = self::get_workspace(workspace_service, None).await?;

    Ok(workspace)
}

pub async fn is_workspace_initialized(db: &DatabaseRwConnectionPools) -> bool {
    crate::onboarding::is_workspace_initialized::get_or_default(&db.read).await
}

// MARK: Get one

pub async fn get_workspace(
    workspace_service: &WorkspaceService,
    auth: Option<&AuthToken>,
) -> Result<Workspace, anyhow::Error> {
    workspace_service.get_workspace(auth).await
}

// MARK: Get/set fields

pub async fn get_workspace_accent_color(
    workspace_service: &WorkspaceService,
    auth: Option<&AuthToken>,
) -> Result<Option<Color>, anyhow::Error> {
    workspace_service.get_workspace_accent_color(auth).await
}
pub async fn set_workspace_accent_color(
    workspace_service: &WorkspaceService,
    auth: &AuthToken,
    accent_color: &Option<Color>,
) -> Result<(), Either<Forbidden, anyhow::Error>> {
    workspace_service
        .set_workspace_accent_color(accent_color, auth)
        .await
}

pub async fn get_workspace_name(
    workspace_service: &WorkspaceService,
    auth: Option<&AuthToken>,
) -> Result<String, anyhow::Error> {
    workspace_service.get_workspace_name(auth).await
}
pub async fn set_workspace_name(
    workspace_service: &WorkspaceService,
    auth: &AuthToken,
    name: &str,
) -> Result<(), Either<Forbidden, anyhow::Error>> {
    workspace_service.set_workspace_name(name, auth).await
}

pub async fn get_workspace_icon(
    workspace_service: &WorkspaceService,
    auth: Option<&AuthToken>,
) -> Result<Option<Avatar>, anyhow::Error> {
    workspace_service.get_workspace_icon(auth).await
}
pub async fn set_workspace_icon(
    workspace_service: &WorkspaceService,
    auth: &AuthToken,
    icon: Avatar,
) -> Result<(), Either<Forbidden, anyhow::Error>> {
    workspace_service.set_workspace_icon(icon, auth).await
}

// MARK: Patch one

#[derive(Debug, Clone)]
#[derive(Validate, serdev::Deserialize)]
#[cfg_attr(feature = "test", derive(Default))]
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
    auth: &AuthToken,
    req: PatchWorkspaceDetailsRequest,
) -> Result<Workspace, Either<Forbidden, anyhow::Error>> {
    workspace_service.patch_workspace(&req.into(), auth).await?;

    let workspace = self::get_workspace(workspace_service, Some(auth)).await?;

    Ok(workspace)
}

// MARK: - Boilerplate

impl From<PatchWorkspaceDetailsRequest> for super::prelude::PatchWorkspaceRequest {
    fn from(req: PatchWorkspaceDetailsRequest) -> Self {
        Self {
            name: req.name,
            accent_color: req.accent_color,
        }
    }
}
