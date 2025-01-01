// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use service::workspace::WorkspaceService;

use crate::{error::Error, guards::LazyGuard};

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceAccentColorResponse {
    pub color: Option<String>,
}

#[rocket::get("/v1/workspace/accent-color")]
pub async fn get_workspace_accent_color_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
) -> Result<Json<GetWorkspaceAccentColorResponse>, Error> {
    let workspace_service = workspace_service.inner?;

    let color = workspace_service.get_workspace_accent_color().await?;

    let response = GetWorkspaceAccentColorResponse { color }.into();
    Ok(response)
}

pub async fn get_workspace_accent_color_route_axum() {
    todo!()
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceAccentColorRequest {
    pub color: String,
}

#[rocket::put("/v1/workspace/accent-color", data = "<req>")]
pub async fn set_workspace_accent_color_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
    req: Json<SetWorkspaceAccentColorRequest>,
) -> Result<Json<GetWorkspaceAccentColorResponse>, Error> {
    let workspace_service = workspace_service.inner?;
    let req = req.into_inner();

    let color = workspace_service
        .set_workspace_accent_color(req.color)
        .await?;

    let response = GetWorkspaceAccentColorResponse { color }.into();
    Ok(response)
}

pub async fn set_workspace_accent_color_route_axum() {
    todo!()
}
