// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use base64::{engine::general_purpose, Engine as _};
use service::workspace::WorkspaceService;

use crate::error::{self, Error};

pub async fn get_workspace_icon_route(
    workspace_service: WorkspaceService,
) -> Result<Json<Option<String>>, Error> {
    let icon = workspace_service.get_workspace_icon_base64().await?;
    Ok(Json(icon))
}

pub async fn set_workspace_icon_route(
    workspace_service: WorkspaceService,
    Json(base64_png): Json<String>,
) -> Result<Json<Option<String>>, Error> {
    let image_data = general_purpose::STANDARD
        .decode(&base64_png)
        .map_err(|err| error::BadRequest {
            reason: format!("Image data should be base64-encoded. Error: {err}"),
        })?;

    workspace_service.set_workspace_icon(image_data).await?;

    Ok(Json(Some(base64_png)))
}
