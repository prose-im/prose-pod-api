// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use base64::{engine::general_purpose, Engine as _};
use rocket::serde::json::Json as JsonRocket;
use serde::{Deserialize, Serialize};
use service::workspace::WorkspaceService;

use crate::{
    error::{self, Error},
    guards::LazyGuard,
};

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceIconResponse {
    pub icon: Option<String>,
}

#[rocket::get("/v1/workspace/icon")]
pub async fn get_workspace_icon_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
) -> Result<JsonRocket<GetWorkspaceIconResponse>, Error> {
    let workspace_service = workspace_service.inner?;

    let icon = workspace_service.get_workspace_icon_base64().await?;

    let response = GetWorkspaceIconResponse { icon }.into();
    Ok(response)
}

pub async fn get_workspace_icon_route_axum(
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

#[rocket::put("/v1/workspace/icon", format = "json", data = "<req>")]
pub async fn set_workspace_icon_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
    req: JsonRocket<SetWorkspaceIconRequest>,
) -> Result<JsonRocket<GetWorkspaceIconResponse>, Error> {
    let workspace_service = workspace_service.inner?;

    let image_data = general_purpose::STANDARD
        .decode(req.image.to_owned())
        .map_err(|err| error::BadRequest {
            reason: format!("Invalid `image` field: data should be base64-encoded. Error: {err}"),
        })?;

    workspace_service.set_workspace_icon(image_data).await?;

    let response = GetWorkspaceIconResponse {
        icon: Some(req.image.to_owned()),
    }
    .into();
    Ok(response)
}

pub async fn set_workspace_icon_route_axum(
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
