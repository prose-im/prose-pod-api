// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, serde::json::Json};
use service::workspace::{Workspace, WorkspaceService};

use crate::{error::Error, guards::LazyGuard};

#[get("/v1/workspace")]
pub async fn get_workspace_route<'r>(
    workspace_service: LazyGuard<WorkspaceService>,
) -> Result<Json<Workspace>, Error> {
    Ok(workspace_service.inner?.get_workspace().await?.into())
}

pub async fn get_workspace_route_axum() {
    todo!()
}
