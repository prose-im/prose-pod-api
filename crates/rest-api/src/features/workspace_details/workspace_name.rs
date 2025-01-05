// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use serde::{Deserialize, Serialize};
use service::workspace::WorkspaceService;

use crate::error::Error;

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceNameResponse {
    pub name: String,
}

pub async fn get_workspace_name_route(
    workspace_service: WorkspaceService,
) -> Result<Json<GetWorkspaceNameResponse>, Error> {
    let name = workspace_service.get_workspace_name().await?;
    Ok(Json(GetWorkspaceNameResponse { name }))
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceNameRequest {
    pub name: String,
}

pub type SetWorkspaceNameResponse = GetWorkspaceNameResponse;

pub async fn set_workspace_name_route(
    workspace_service: WorkspaceService,
    Json(req): Json<SetWorkspaceNameRequest>,
) -> Result<Json<SetWorkspaceNameResponse>, Error> {
    let name = workspace_service.set_workspace_name(req.name).await?;
    Ok(Json(SetWorkspaceNameResponse { name }))
}
