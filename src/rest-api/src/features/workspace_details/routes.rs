// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    http::{header::IF_MATCH, HeaderValue, StatusCode},
    response::NoContent,
    Json,
};
use axum_extra::{either::Either, headers::IfMatch, TypedHeader};
use service::{
    models::{Avatar, AvatarOwned, Color},
    workspace::{
        workspace_controller::{self, PatchWorkspaceDetailsRequest},
        Workspace, WorkspaceService,
    },
};
use validator::Validate;

use crate::{
    error::{Error, PreconditionRequired},
    responders::{self, Created},
};

use super::WORKSPACE_ROUTE;

// MARK: INIT WORKSPACE

#[derive(Debug, Clone)]
#[derive(Validate, serdev::Deserialize)]
#[serde(validate = "Validate::validate")]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct InitWorkspaceRequest {
    /// Organization name.
    #[validate(length(min = 1, max = 48), non_control_character)]
    pub name: String,

    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    #[validate(skip)] // NOTE: Already parsed.
    pub accent_color: Option<Color>,
}

pub async fn init_workspace_route(
    ref workspace_service: WorkspaceService,
    Json(req): Json<InitWorkspaceRequest>,
) -> Result<Created<Workspace>, Error> {
    let workspace = workspace_controller::init_workspace(workspace_service, req).await?;

    let resource_uri = WORKSPACE_ROUTE;
    Ok(Created {
        location: HeaderValue::from_static(resource_uri),
        body: workspace,
    })
}

// MARK: GET ONE

pub async fn get_workspace_route(
    ref workspace_service: WorkspaceService,
) -> Result<Json<Workspace>, Error> {
    match workspace_controller::get_workspace(workspace_service).await? {
        workspace => Ok(Json(workspace)),
    }
}

pub async fn is_workspace_initialized_route(
    TypedHeader(if_match): TypedHeader<IfMatch>,
    workspace_service: Option<WorkspaceService>,
) -> Result<StatusCode, Error> {
    if if_match != IfMatch::any() {
        Err(Error::from(PreconditionRequired {
            comment: format!("Missing header: '{IF_MATCH}'."),
        }))
    } else {
        match workspace_service {
            // NOTE: `WorkspaceService` needs the Server config to be initialized.
            //   In order to check the `If-Match` precondition first (the result
            //   wouldn’t make sense otherwise) we have to make `WorkspaceService`
            //   optional.
            Some(ref s) if workspace_controller::is_workspace_initialized(s).await? => {
                Ok(StatusCode::NO_CONTENT)
            }
            _ => Ok(StatusCode::PRECONDITION_FAILED),
        }
    }
}

// MARK: GET/SET FIELDS

pub async fn get_workspace_accent_color_route(
    ref workspace_service: WorkspaceService,
) -> Result<Json<Option<Color>>, Error> {
    match workspace_controller::get_workspace_accent_color(workspace_service).await? {
        accent_color => Ok(Json(accent_color)),
    }
}
pub async fn set_workspace_accent_color_route(
    ref workspace_service: WorkspaceService,
    Json(accent_color): Json<Option<Color>>,
) -> Result<Json<Option<Color>>, Error> {
    match workspace_controller::set_workspace_accent_color(workspace_service, accent_color).await? {
        accent_color => Ok(Json(accent_color)),
    }
}

pub async fn get_workspace_name_route(
    ref workspace_service: WorkspaceService,
) -> Result<Json<String>, Error> {
    match workspace_controller::get_workspace_name(workspace_service).await? {
        name => Ok(Json(name)),
    }
}
pub async fn set_workspace_name_route(
    ref workspace_service: WorkspaceService,
    Json(name): Json<String>,
) -> Result<Json<String>, Error> {
    match workspace_controller::set_workspace_name(workspace_service, name).await? {
        name => Ok(Json(name)),
    }
}

pub async fn get_workspace_icon_route(
    ref workspace_service: WorkspaceService,
) -> Result<Either<responders::Avatar, NoContent>, Error> {
    match workspace_controller::get_workspace_icon(workspace_service).await? {
        Some(icon) => Ok(Either::E1(responders::Avatar(icon))),
        None => Ok(Either::E2(NoContent)),
    }
}
pub async fn get_workspace_icon_json_route(
    ref workspace_service: WorkspaceService,
) -> Result<Json<Option<AvatarOwned>>, Error> {
    match workspace_controller::get_workspace_icon(workspace_service).await? {
        icon => Ok(Json(icon)),
    }
}
pub async fn set_workspace_icon_route<'a>(
    ref workspace_service: WorkspaceService,
    icon: Avatar<'a>,
) -> Result<NoContent, Error> {
    workspace_controller::set_workspace_icon(workspace_service, icon).await?;
    Ok(NoContent)
}

// MARK: PATCH ONE

pub async fn patch_workspace_route(
    ref workspace_service: WorkspaceService,
    Json(req): Json<PatchWorkspaceDetailsRequest>,
) -> Result<Json<Workspace>, Error> {
    match workspace_controller::patch_workspace(workspace_service, req).await? {
        workspace => Ok(Json(workspace)),
    }
}

// MARK: BOILERPLATE

impl Into<Workspace> for InitWorkspaceRequest {
    fn into(self) -> Workspace {
        Workspace {
            name: self.name,
            accent_color: self.accent_color,
            icon: None,
        }
    }
}
