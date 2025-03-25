// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use serde::{Deserialize, Serialize};
use service::{
    workspace::{Workspace, WorkspaceService},
    xmpp::VCard,
};

use crate::error::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchWorkspaceDetailsRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::forms::deserialize_some")]
    pub accent_color: Option<Option<String>>,
}

pub async fn patch_workspace_route(
    workspace_service: WorkspaceService,
    Json(req): Json<PatchWorkspaceDetailsRequest>,
) -> Result<Json<Workspace>, Error> {
    let mut workspace = workspace_service.get_workspace().await?;

    if let Some(name) = req.name {
        workspace.name = name
    }
    if let Some(accent_color) = req.accent_color {
        workspace.accent_color = accent_color
    }

    let vcard = VCard::from(workspace.clone());
    workspace_service.set_workspace_vcard(&vcard).await?;

    Ok(Json(workspace))
}
