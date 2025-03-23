// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use serde::{Deserialize, Serialize};
use service::workspace::WorkspaceService;

use crate::error::Error;

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceAccentColorResponse {
    pub color: Option<String>,
}

pub async fn get_workspace_accent_color_route(
    workspace_service: WorkspaceService,
) -> Result<Json<GetWorkspaceAccentColorResponse>, Error> {
    let color = workspace_service.get_workspace_accent_color().await?;
    Ok(Json(GetWorkspaceAccentColorResponse { color }))
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceAccentColorRequest {
    pub color: String,
}

pub async fn set_workspace_accent_color_route(
    workspace_service: WorkspaceService,
    Json(req): Json<SetWorkspaceAccentColorRequest>,
) -> Result<Json<GetWorkspaceAccentColorResponse>, Error> {
    let color = workspace_service
        .set_workspace_accent_color(req.color)
        .await?;
    Ok(Json(GetWorkspaceAccentColorResponse { color: Some(color) }))
}
