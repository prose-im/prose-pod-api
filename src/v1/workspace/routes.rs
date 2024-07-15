// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{
    fs::TempFile,
    response::status::NoContent,
    serde::json::Json,
    tokio::io,
    {get, put},
};
use serde::{Deserialize, Serialize};
use service::{controllers::workspace_controller::WorkspaceController, model::Workspace};

use crate::{
    error::{self, Error},
    guards::LazyGuard,
    v1::R,
};

#[get("/v1/workspace")]
pub async fn get_workspace<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> R<Workspace> {
    Ok(workspace_controller.inner?.get_workspace().await?.into())
}

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceNameResponse {
    pub name: String,
}

#[get("/v1/workspace/name")]
pub(super) async fn get_workspace_name<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> R<GetWorkspaceNameResponse> {
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
pub(super) async fn set_workspace_name<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
    req: Json<SetWorkspaceNameRequest>,
) -> R<SetWorkspaceNameResponse> {
    let workspace_controller = workspace_controller.inner?;
    let req = req.into_inner();

    let name = workspace_controller.set_workspace_name(req.name).await?;

    let response = SetWorkspaceNameResponse { name }.into();
    Ok(response)
}

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceIconResponse {
    pub url: Option<String>,
}

#[get("/v1/workspace/icon")]
pub(super) async fn get_workspace_icon<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> R<GetWorkspaceIconResponse> {
    let workspace_controller = workspace_controller.inner?;

    let url = workspace_controller.get_workspace_icon().await?;

    let response = GetWorkspaceIconResponse { url }.into();
    Ok(response)
}

#[put("/v1/workspace/icon", format = "plain", data = "<string>", rank = 1)]
pub(super) async fn set_workspace_icon_string<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
    string: String,
) -> R<GetWorkspaceIconResponse> {
    let workspace_controller = workspace_controller.inner?;

    let url = workspace_controller
        .set_workspace_icon_string(string)
        .await?;

    let response = GetWorkspaceIconResponse { url }.into();
    Ok(response)
}

#[put("/v1/workspace/icon", format = "plain", data = "<image>", rank = 2)]
pub(super) async fn set_workspace_icon_file(image: TempFile<'_>) -> R<GetWorkspaceIconResponse> {
    let mut stream = image.open().await?;
    let mut data: Vec<u8> = Vec::new();
    io::copy(&mut stream, &mut data).await?;

    Err(error::NotImplemented("Set workspace icon from a file").into())
}

#[get("/v1/workspace/details-card")]
pub(super) fn get_workspace_details_card() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Get workspace vCard").into())
}

#[put("/v1/workspace/details-card")]
pub(super) fn set_workspace_details_card() -> Result<NoContent, Error> {
    Err(error::NotImplemented("Set workspace vCard").into())
}

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceAccentColorResponse {
    pub color: Option<String>,
}

#[get("/v1/workspace/accent-color")]
pub(super) async fn get_workspace_accent_color<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> R<GetWorkspaceAccentColorResponse> {
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
pub(super) async fn set_workspace_accent_color<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
    req: Json<SetWorkspaceAccentColorRequest>,
) -> R<GetWorkspaceAccentColorResponse> {
    let workspace_controller = workspace_controller.inner?;
    let req = req.into_inner();

    let color = workspace_controller
        .set_workspace_accent_color(req.color)
        .await?;

    let response = GetWorkspaceAccentColorResponse { color }.into();
    Ok(response)
}
