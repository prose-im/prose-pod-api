// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use service::workspace::WorkspaceService;

use crate::error::Error;

pub async fn get_workspace_name_route(
    workspace_service: WorkspaceService,
) -> Result<Json<String>, Error> {
    let name = workspace_service.get_workspace_name().await?;
    Ok(Json(name))
}

pub async fn set_workspace_name_route(
    workspace_service: WorkspaceService,
    Json(name): Json<String>,
) -> Result<Json<String>, Error> {
    let name = workspace_service.set_workspace_name(name).await?;
    Ok(Json(name))
}
