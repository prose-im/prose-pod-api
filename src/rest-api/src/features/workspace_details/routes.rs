// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::State,
    http::{
        header::{ACCEPT, IF_MATCH},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::NoContent,
    Json,
};
use axum_extra::{either::Either, headers::IfMatch, TypedHeader};
use mime::APPLICATION_JSON;
use service::{
    auth::AuthToken,
    models::{Avatar, Color},
    workspace::{
        workspace_controller::{self, PatchWorkspaceDetailsRequest},
        Workspace, WorkspaceService,
    },
};
use validator::Validate;

use crate::{
    error::{Error, PreconditionRequired},
    responders::{self, Created},
    util::headers_ext::HeaderValueExt as _,
    AppState,
};

use super::WORKSPACE_ROUTE;

// MARK: Init workspace

#[derive(Debug, Clone)]
#[derive(Validate, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
#[cfg_attr(feature = "test", derive(serdev::Serialize))]
pub struct InitWorkspaceRequest {
    /// Organization name.
    #[validate(length(min = 1, max = 48), non_control_character)]
    pub name: String,

    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    #[validate(skip)] // NOTE: Already parsed.
    #[serde(default)]
    pub accent_color: Option<Color>,
}

// COMPAT: This route will disappear because the Workspace is now initialized
//   by default, but until then it’ll map to `PATCH /v1/workspace`, with a
//   check that ensures the name is set (not to change behavior).
pub async fn init_workspace_route(
    State(AppState { ref db, .. }): State<AppState>,
    ref workspace_service: WorkspaceService,
    Json(req): Json<InitWorkspaceRequest>,
) -> Result<Created<Workspace>, Error> {
    let workspace = workspace_controller::init_workspace(db, workspace_service, req).await?;

    let resource_uri = WORKSPACE_ROUTE;
    Ok(Created {
        location: HeaderValue::from_static(resource_uri),
        body: workspace,
    })
}

// MARK: Get one

pub async fn get_workspace_route(
    ref workspace_service: WorkspaceService,
    auth: Option<AuthToken>,
) -> Result<Json<Workspace>, Error> {
    match workspace_controller::get_workspace(workspace_service, auth.as_ref()).await? {
        workspace => Ok(Json(workspace)),
    }
}

pub async fn is_workspace_initialized_route(
    State(AppState { ref db, .. }): State<AppState>,
    TypedHeader(if_match): TypedHeader<IfMatch>,
) -> Result<StatusCode, Error> {
    if if_match != IfMatch::any() {
        Err(Error::from(PreconditionRequired {
            comment: format!("Missing header: '{IF_MATCH}'."),
        }))
    } else if workspace_controller::is_workspace_initialized(db).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::PRECONDITION_FAILED)
    }
}

// MARK: Get/set fields

pub async fn get_workspace_accent_color_route(
    ref workspace_service: WorkspaceService,
    auth: Option<AuthToken>,
) -> Result<Json<Option<Color>>, Error> {
    match workspace_controller::get_workspace_accent_color(workspace_service, auth.as_ref()).await?
    {
        accent_color => Ok(Json(accent_color)),
    }
}
pub async fn set_workspace_accent_color_route(
    ref workspace_service: WorkspaceService,
    ref auth: AuthToken,
    Json(accent_color): Json<Option<Color>>,
) -> Result<Json<Option<Color>>, Error> {
    match workspace_controller::set_workspace_accent_color(workspace_service, auth, &accent_color)
        .await?
    {
        () => Ok(Json(accent_color)),
    }
}

pub async fn get_workspace_name_route(
    ref workspace_service: WorkspaceService,
    auth: Option<AuthToken>,
) -> Result<Json<String>, Error> {
    match workspace_controller::get_workspace_name(workspace_service, auth.as_ref()).await? {
        name => Ok(Json(name)),
    }
}
pub async fn set_workspace_name_route(
    ref workspace_service: WorkspaceService,
    ref auth: AuthToken,
    Json(name): Json<String>,
) -> Result<Json<String>, Error> {
    match workspace_controller::set_workspace_name(workspace_service, auth, &name).await? {
        () => Ok(Json(name)),
    }
}

pub async fn get_workspace_icon_route(
    workspace_service: WorkspaceService,
    headers: HeaderMap,
    auth: Option<AuthToken>,
) -> Either<Result<Either<responders::Avatar, NoContent>, Error>, Result<Json<Option<Avatar>>, Error>>
{
    match headers.get(ACCEPT) {
        Some(ct) if ct.starts_with(APPLICATION_JSON.essence_str()) => {
            Either::E2(get_workspace_icon_json_route_(workspace_service, auth).await)
        }
        _ => Either::E1(get_workspace_icon_route_(workspace_service, auth).await),
    }
}
async fn get_workspace_icon_route_(
    ref workspace_service: WorkspaceService,
    auth: Option<AuthToken>,
) -> Result<Either<responders::Avatar, NoContent>, Error> {
    match workspace_controller::get_workspace_icon(workspace_service, auth.as_ref()).await? {
        Some(icon) => Ok(Either::E1(responders::Avatar(icon))),
        None => Ok(Either::E2(NoContent)),
    }
}
async fn get_workspace_icon_json_route_(
    ref workspace_service: WorkspaceService,
    auth: Option<AuthToken>,
) -> Result<Json<Option<Avatar>>, Error> {
    match workspace_controller::get_workspace_icon(workspace_service, auth.as_ref()).await? {
        icon => Ok(Json(icon)),
    }
}
pub async fn set_workspace_icon_route(
    ref workspace_service: WorkspaceService,
    ref auth: AuthToken,
    icon: Avatar,
) -> Result<NoContent, Error> {
    workspace_controller::set_workspace_icon(workspace_service, auth, icon).await?;
    Ok(NoContent)
}

// MARK: Patch one

pub async fn patch_workspace_route(
    ref workspace_service: WorkspaceService,
    ref auth: AuthToken,
    Json(req): Json<PatchWorkspaceDetailsRequest>,
) -> Result<Json<Workspace>, Error> {
    match workspace_controller::patch_workspace(workspace_service, auth, req).await? {
        workspace => Ok(Json(workspace)),
    }
}

// MARK: - Boilerplate

impl From<InitWorkspaceRequest> for service::prose_pod_server_api::InitWorkspaceRequest {
    fn from(req: InitWorkspaceRequest) -> Self {
        Self {
            name: req.name,
            accent_color: req.accent_color,
        }
    }
}
