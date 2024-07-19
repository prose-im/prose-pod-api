// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use base64::{engine::general_purpose, Engine as _};
use rocket::{
    response::status::NoContent,
    serde::json::Json,
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
    pub icon: Option<String>,
}

#[get("/v1/workspace/icon")]
pub(super) async fn get_workspace_icon<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> R<GetWorkspaceIconResponse> {
    let workspace_controller = workspace_controller.inner?;

    let avatar_data = workspace_controller.get_workspace_icon().await?;
    let icon = avatar_data.map(|d| d.base64().into_owned());

    let response = GetWorkspaceIconResponse { icon }.into();
    Ok(response)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetWorkspaceIconRequest {
    // Base64 encoded image
    pub image: String,
}

#[put("/v1/workspace/icon", format = "json", data = "<req>")]
pub(super) async fn set_workspace_icon<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
    req: Json<SetWorkspaceIconRequest>,
) -> R<GetWorkspaceIconResponse> {
    let workspace_controller = workspace_controller.inner?;

    let image_data = general_purpose::STANDARD
        .decode(req.image.to_owned())
        .map_err(|err| error::BadRequest {
            reason: format!("Invalid `image` field: data should be base64-encoded. Error: {err}"),
        })?;

    workspace_controller.set_workspace_icon(image_data).await?;

    let response = GetWorkspaceIconResponse {
        icon: Some(req.image.to_owned()),
    }
    .into();
    Ok(response)
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
