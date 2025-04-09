// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    http::{header::IF_MATCH, StatusCode},
    Json,
};
use axum_extra::{headers::IfMatch, TypedHeader};
use service::workspace::{Workspace, WorkspaceService};

use crate::error::{Error, PreconditionRequired};

pub async fn get_workspace_route(
    workspace_service: WorkspaceService,
) -> Result<Json<Workspace>, Error> {
    let workspace = workspace_service.get_workspace().await?;
    Ok(Json(workspace))
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
        Ok(match workspace_service {
            // NOTE: `WorkspaceService` needs the Server config to be initialized.
            //   In order to check the `If-Match` precondition first (the result
            //   wouldn’t make sense otherwise) we have to make `WorkspaceService`
            //   optional.
            Some(s) if s.is_workspace_initialized().await? => StatusCode::NO_CONTENT,
            _ => StatusCode::PRECONDITION_FAILED,
        })
    }
}
