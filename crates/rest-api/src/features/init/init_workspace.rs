// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use service::{
    config::AppConfig,
    controllers::init_controller::{InitController, WorkspaceCreateForm},
    model::ServerConfig,
    services::{secrets_store::SecretsStore, xmpp_service::XmppServiceInner},
};

use crate::{guards::LazyGuard, responders::Created};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitWorkspaceRequest {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitWorkspaceResponse {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

#[put("/v1/workspace", format = "json", data = "<req>")]
pub async fn init_workspace_route<'r>(
    init_controller: LazyGuard<InitController<'r>>,
    app_config: &State<AppConfig>,
    secrets_store: &State<SecretsStore>,
    xmpp_service: &State<XmppServiceInner>,
    server_config: LazyGuard<ServerConfig>,
    req: Json<InitWorkspaceRequest>,
) -> Created<InitWorkspaceResponse> {
    let init_controller = init_controller.inner?;
    let server_config = server_config.inner?;
    let req = req.into_inner();

    let workspace = init_controller
        .init_workspace(
            app_config,
            secrets_store,
            xmpp_service,
            &server_config,
            req.clone(),
        )
        .await?;

    let response = InitWorkspaceResponse {
        name: req.name,
        accent_color: workspace.accent_color,
    };

    let resource_uri = uri!(crate::features::workspace_details::get_workspace_route).to_string();
    Ok(status::Created::new(resource_uri).body(response.into()))
}

// BOILERPLATE

impl Into<WorkspaceCreateForm> for InitWorkspaceRequest {
    fn into(self) -> WorkspaceCreateForm {
        WorkspaceCreateForm {
            name: self.name,
            accent_color: Some(self.accent_color),
        }
    }
}
