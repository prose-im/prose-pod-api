// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, serde::json::Json};
use service::workspace::{Workspace, WorkspaceController};

use crate::{error::Error, guards::LazyGuard};

#[get("/v1/workspace")]
pub async fn get_workspace_route<'r>(
    workspace_controller: LazyGuard<WorkspaceController<'r>>,
) -> Result<Json<Workspace>, Error> {
    Ok(workspace_controller.inner?.get_workspace().await?.into())
}
