// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use service::workspace::WorkspaceService;

use crate::error::Error;

pub async fn get_workspace_accent_color_route(
    workspace_service: WorkspaceService,
) -> Result<Json<Option<String>>, Error> {
    let accent_color = workspace_service.get_workspace_accent_color().await?;
    Ok(Json(accent_color))
}

pub async fn set_workspace_accent_color_route(
    workspace_service: WorkspaceService,
    Json(accent_color): Json<Option<String>>,
) -> Result<Json<Option<String>>, Error> {
    let accent_color = workspace_service
        .set_workspace_accent_color(accent_color)
        .await?;
    Ok(Json(accent_color))
}
