// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use service::workspace::{Workspace, WorkspaceService};

use crate::error::Error;

pub async fn get_workspace_route(
    workspace_service: WorkspaceService,
) -> Result<Json<Workspace>, Error> {
    let workspace = workspace_service.get_workspace().await?;
    Ok(Json(workspace))
}
