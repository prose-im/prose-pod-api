// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::server_config;
use rocket::fs::TempFile;
use rocket::serde::json::Json;
use rocket::{get, put};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::sea_orm::{ActiveModelTrait as _, Set};
use utoipa::ToSchema;

use crate::error::Error;
use crate::guards::{Db, LazyGuard, ServerConfig};

pub type R<T> = Result<Json<T>, Error>;

#[derive(Serialize, Deserialize, ToSchema)]
#[cfg_attr(test, derive(Debug))]
pub struct GetWorkspaceNameResponse {
    #[schema(example = "Crisp")]
    pub name: String,
}

/// Get the name of your workspace.
#[utoipa::path(
    tag = "Workspace",
    responses(
        (status = 200, description = "Success", body = GetWorkspaceNameResponse)
    )
)]
#[get("/v1/workspace/name")]
pub(super) fn get_workspace_name(
    server_config: LazyGuard<ServerConfig>,
) -> R<GetWorkspaceNameResponse> {
    let server_config = server_config.inner?.model();
    let response = GetWorkspaceNameResponse {
        name: server_config.workspace_name,
    }
    .into();

    Ok(response)
}

#[derive(Serialize, Deserialize, ToSchema)]
#[cfg_attr(test, derive(Debug))]
pub struct SetWorkspaceNameRequest {
    #[schema(example = "Crisp")]
    pub name: String,
}

pub type SetWorkspaceNameResponse = GetWorkspaceNameResponse;

/// Set the name of your workspace.
#[utoipa::path(
    tag = "Workspace",
    responses(
        // `utoipa` does not resolve type aliases,
        // we need to set `body` to its concrete implementation.
        (status = 200, description = "Success", body = GetWorkspaceNameResponse)
    )
)]
#[put("/v1/workspace/name", format = "json", data = "<req>")]
pub(super) async fn set_workspace_name(
    conn: Connection<'_, Db>,
    server_config: LazyGuard<ServerConfig>,
    req: Json<SetWorkspaceNameRequest>,
) -> R<SetWorkspaceNameResponse> {
    let db = conn.into_inner();
    let server_config = server_config.inner?.model();

    let mut active: server_config::ActiveModel = server_config.into();
    active.workspace_name = Set(req.name.clone());
    let server_config = active.update(db).await?;

    let response = SetWorkspaceNameResponse {
        name: server_config.workspace_name,
    }
    .into();

    Ok(response)
}

#[derive(Serialize, Deserialize, ToSchema)]
#[cfg_attr(test, derive(Debug))]
pub struct GetWorkspaceIconResponse {
    pub url: Option<String>,
}

/// Get the icon of your workspace.
#[utoipa::path(
    tag = "Workspace",
    responses(
        (status = 200, description = "Success", body = GetWorkspaceIconResponse)
    )
)]
#[get("/v1/workspace/icon")]
pub(super) fn get_workspace_icon(
    server_config: LazyGuard<ServerConfig>,
) -> R<GetWorkspaceIconResponse> {
    let server_config = server_config.inner?.model();
    let response = GetWorkspaceIconResponse {
        url: server_config.workspace_icon_url,
    }
    .into();

    Ok(response)
}

// NOTE: Documentation overriden in <./openapi_extension.rs>.
#[put("/v1/workspace/icon", format = "plain", data = "<string>", rank = 1)]
pub(super) async fn set_workspace_icon_string(
    conn: Connection<'_, Db>,
    server_config: LazyGuard<ServerConfig>,
    string: String,
) -> R<GetWorkspaceIconResponse> {
    let db = conn.into_inner();
    let server_config = server_config.inner?.model();

    // TODO: Validate `string`
    let mut active: server_config::ActiveModel = server_config.into();
    active.workspace_icon_url = Set(Some(string));
    let server_config = active.update(db).await?;

    let response = GetWorkspaceIconResponse {
        url: server_config.workspace_icon_url,
    }
    .into();

    Ok(response)
}

// NOTE: Documentation overriden in <./openapi_extension.rs>.
#[put("/v1/workspace/icon", format = "plain", data = "<_image>", rank = 2)]
pub(super) fn set_workspace_icon_file(mut _image: TempFile<'_>) -> Json<GetWorkspaceIconResponse> {
    todo!()
}

/// Get the details card of your workspace.
#[utoipa::path(
    tag = "Workspace",
    responses(
        (status = 200, description = "Success", content_type = "text/vcard", body = String),
    ),
)]
#[get("/v1/workspace/details-card")]
pub(super) fn get_workspace_details_card() -> String {
    todo!()
}

/// Set the details card of your workspace.
#[utoipa::path(
    tag = "Workspace",
    request_body(content = String, content_type = "text/vcard"),
    responses(
        (status = 200, description = "Success"),
    ),
)]
#[put("/v1/workspace/details-card", data = "<_vcard>")]
pub(super) fn set_workspace_details_card(_vcard: String) {
    todo!()
}

#[derive(Serialize, Deserialize, ToSchema)]
#[cfg_attr(test, derive(Debug))]
pub struct GetWorkspaceAccentColorResponse {
    #[schema(example = "#4233BE")]
    pub color: Option<String>,
}

/// Get the accent color of your workspace.
#[utoipa::path(
    tag = "Workspace",
    responses(
        (status = 200, description = "Success", body = GetWorkspaceAccentColorResponse),
    ),
)]
#[get("/v1/workspace/accent-color")]
pub(super) fn get_workspace_accent_color(
    server_config: LazyGuard<ServerConfig>,
) -> R<GetWorkspaceAccentColorResponse> {
    let server_config = server_config.inner?.model();
    let response = GetWorkspaceAccentColorResponse {
        color: server_config.workspace_accent_color,
    }
    .into();

    Ok(response)
}

#[derive(Serialize, Deserialize, ToSchema)]
#[cfg_attr(test, derive(Debug))]
pub struct SetWorkspaceAccentColorRequest {
    #[schema(example = "#226EF2")]
    pub color: String,
}

/// Set the accent color of your workspace.
#[utoipa::path(
    tag = "Workspace",
    responses(
        (status = 200, description = "Success", body = GetWorkspaceAccentColorResponse)
    )
)]
#[put("/v1/workspace/accent-color", data = "<req>")]
pub(super) async fn set_workspace_accent_color(
    conn: Connection<'_, Db>,
    server_config: LazyGuard<ServerConfig>,
    req: Json<SetWorkspaceAccentColorRequest>,
) -> R<GetWorkspaceAccentColorResponse> {
    let db = conn.into_inner();
    let server_config = server_config.inner?.model();

    // TODO: Validate `string`
    let mut active: server_config::ActiveModel = server_config.into();
    active.workspace_accent_color = Set(Some(req.color.clone()));
    let server_config = active.update(db).await?;

    let response = GetWorkspaceAccentColorResponse {
        color: server_config.workspace_accent_color,
    }
    .into();

    Ok(response)
}
