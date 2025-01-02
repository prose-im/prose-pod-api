// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use service::workspace::{Workspace, WorkspaceService};

use crate::{error::Error, guards::LazyGuard};

#[rocket::get("/v1/workspace")]
pub async fn get_workspace_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
) -> Result<rocket::serde::json::Json<Workspace>, Error> {
    Ok(workspace_service.inner?.get_workspace().await?.into())
}

pub async fn get_workspace_route_axum(
    workspace_service: WorkspaceService,
) -> Result<Json<Workspace>, Error> {
    let workspace = workspace_service.get_workspace().await?;
    Ok(Json(workspace))
}
