// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::fs::TempFile;
use rocket::response::status::NoContent;
use rocket::serde::json::Json;
use rocket::tokio::io;
use rocket::{get, put};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::controllers::workspace_controller::WorkspaceController;
use service::model::Workspace;

use crate::error::{self, Error};
use crate::guards::{Db, LazyGuard};
use crate::v1::R;

#[get("/v1/workspace")]
pub async fn get_workspace(conn: Connection<'_, Db>) -> R<Workspace> {
    match WorkspaceController::get_workspace(conn.into_inner()).await? {
        Some(workspace) => Ok(workspace.into()),
        None => Err(error::WorkspaceNotInitialized.into()),
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceNameResponse {
    pub name: String,
}

#[get("/v1/workspace/name")]
pub(super) async fn get_workspace_name(
    workspace: LazyGuard<Workspace>,
) -> R<GetWorkspaceNameResponse> {
    let workspace = workspace.inner?;

    let name = WorkspaceController::get_workspace_name(workspace).await;

    let response = GetWorkspaceNameResponse { name }.into();
    Ok(response)
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceNameRequest {
    pub name: String,
}

pub type SetWorkspaceNameResponse = GetWorkspaceNameResponse;

#[put("/v1/workspace/name", format = "json", data = "<req>")]
pub(super) async fn set_workspace_name(
    conn: Connection<'_, Db>,
    workspace: LazyGuard<Workspace>,
    req: Json<SetWorkspaceNameRequest>,
) -> R<SetWorkspaceNameResponse> {
    let db = conn.into_inner();
    let workspace = workspace.inner?;
    let req = req.into_inner();

    let name = WorkspaceController::set_workspace_name(db, workspace, req.name).await?;

    let response = SetWorkspaceNameResponse { name }.into();
    Ok(response)
}

#[derive(Serialize, Deserialize)]
pub struct GetWorkspaceIconResponse {
    pub url: Option<String>,
}

#[get("/v1/workspace/icon")]
pub(super) fn get_workspace_icon(workspace: LazyGuard<Workspace>) -> R<GetWorkspaceIconResponse> {
    let workspace = workspace.inner?;

    let url = WorkspaceController::get_workspace_icon(workspace);

    let response = GetWorkspaceIconResponse { url }.into();
    Ok(response)
}

#[put("/v1/workspace/icon", format = "plain", data = "<string>", rank = 1)]
pub(super) async fn set_workspace_icon_string(
    conn: Connection<'_, Db>,
    workspace: LazyGuard<Workspace>,
    string: String,
) -> R<GetWorkspaceIconResponse> {
    let db = conn.into_inner();
    let workspace = workspace.inner?;

    let url = WorkspaceController::set_workspace_icon_string(db, workspace, string).await?;

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
pub(super) fn get_workspace_accent_color(
    workspace: LazyGuard<Workspace>,
) -> R<GetWorkspaceAccentColorResponse> {
    let workspace = workspace.inner?;

    let color = WorkspaceController::get_workspace_accent_color(workspace);

    let response = GetWorkspaceAccentColorResponse { color }.into();
    Ok(response)
}

#[derive(Serialize, Deserialize)]
pub struct SetWorkspaceAccentColorRequest {
    pub color: String,
}

#[put("/v1/workspace/accent-color", data = "<req>")]
pub(super) async fn set_workspace_accent_color(
    conn: Connection<'_, Db>,
    workspace: LazyGuard<Workspace>,
    req: Json<SetWorkspaceAccentColorRequest>,
) -> R<GetWorkspaceAccentColorResponse> {
    let db = conn.into_inner();
    let workspace = workspace.inner?;
    let req = req.into_inner();

    let color = WorkspaceController::set_workspace_accent_color(db, workspace, req.color).await?;

    let response = GetWorkspaceAccentColorResponse { color }.into();
    Ok(response)
}
