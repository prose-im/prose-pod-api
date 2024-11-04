// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use service::controllers::workspace_controller::WorkspaceController;

use crate::{error::Error, guards::LazyGuard};

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceAccentColorResponse {
    pub color: Option<String>,
}

#[get("/v1/workspace/accent-color")]
pub async fn get_workspace_accent_color_route<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> Result<Json<GetWorkspaceAccentColorResponse>, Error> {
    let workspace_controller = workspace_controller.inner?;

    let color = workspace_controller.get_workspace_accent_color().await?;

    let response = GetWorkspaceAccentColorResponse { color }.into();
    Ok(response)
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceAccentColorRequest {
    pub color: String,
}

#[put("/v1/workspace/accent-color", data = "<req>")]
pub async fn set_workspace_accent_color_route<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
    req: Json<SetWorkspaceAccentColorRequest>,
) -> Result<Json<GetWorkspaceAccentColorResponse>, Error> {
    let workspace_controller = workspace_controller.inner?;
    let req = req.into_inner();

    let color = workspace_controller
        .set_workspace_accent_color(req.color)
        .await?;

    let response = GetWorkspaceAccentColorResponse { color }.into();
    Ok(response)
}
