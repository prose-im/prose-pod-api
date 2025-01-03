// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use service::workspace::WorkspaceService;

use crate::error::{self, Error};

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceIconResponse {
    pub icon: Option<String>,
}

pub async fn get_workspace_icon_route(
    workspace_service: WorkspaceService,
) -> Result<Json<GetWorkspaceIconResponse>, Error> {
    let icon = workspace_service.get_workspace_icon_base64().await?;
    Ok(Json(GetWorkspaceIconResponse { icon }))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetWorkspaceIconRequest {
    // Base64 encoded image
    pub image: String,
}

pub async fn set_workspace_icon_route(
    workspace_service: WorkspaceService,
    req: Json<SetWorkspaceIconRequest>,
) -> Result<Json<GetWorkspaceIconResponse>, Error> {
    let image_data = general_purpose::STANDARD
        .decode(req.image.to_owned())
        .map_err(|err| error::BadRequest {
            reason: format!("Invalid `image` field: data should be base64-encoded. Error: {err}"),
        })?;

    workspace_service.set_workspace_icon(image_data).await?;

    Ok(Json(GetWorkspaceIconResponse {
        icon: Some(req.image.to_owned()),
    }))
}
