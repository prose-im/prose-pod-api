// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use service::workspace::WorkspaceController;

use crate::{error::Error, guards::LazyGuard};

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceNameResponse {
    pub name: String,
}

#[get("/v1/workspace/name")]
pub async fn get_workspace_name_route<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> Result<Json<GetWorkspaceNameResponse>, Error> {
    let workspace_controller = workspace_controller.inner?;

    let name = workspace_controller.get_workspace_name().await?;

    let response = GetWorkspaceNameResponse { name }.into();
    Ok(response)
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceNameRequest {
    pub name: String,
}

pub type SetWorkspaceNameResponse = GetWorkspaceNameResponse;

#[put("/v1/workspace/name", format = "json", data = "<req>")]
pub async fn set_workspace_name_route<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
    req: Json<SetWorkspaceNameRequest>,
) -> Result<Json<SetWorkspaceNameResponse>, Error> {
    let workspace_controller = workspace_controller.inner?;
    let req = req.into_inner();

    let name = workspace_controller.set_workspace_name(req.name).await?;

    let response = SetWorkspaceNameResponse { name }.into();
    Ok(response)
}
