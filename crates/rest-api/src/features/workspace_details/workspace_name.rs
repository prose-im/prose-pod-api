// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use service::workspace::WorkspaceService;

use crate::{error::Error, guards::LazyGuard};

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceNameResponse {
    pub name: String,
}

#[rocket::get("/v1/workspace/name")]
pub async fn get_workspace_name_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
) -> Result<Json<GetWorkspaceNameResponse>, Error> {
    let workspace_service = workspace_service.inner?;

    let name = workspace_service.get_workspace_name().await?;

    let response = GetWorkspaceNameResponse { name }.into();
    Ok(response)
}

pub async fn get_workspace_name_route_axum() {
    todo!()
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceNameRequest {
    pub name: String,
}

pub type SetWorkspaceNameResponse = GetWorkspaceNameResponse;

#[rocket::put("/v1/workspace/name", format = "json", data = "<req>")]
pub async fn set_workspace_name_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
    req: Json<SetWorkspaceNameRequest>,
) -> Result<Json<SetWorkspaceNameResponse>, Error> {
    let workspace_service = workspace_service.inner?;
    let req = req.into_inner();

    let name = workspace_service.set_workspace_name(req.name).await?;

    let response = SetWorkspaceNameResponse { name }.into();
    Ok(response)
}

pub async fn set_workspace_name_route_axum() {
    todo!()
}
