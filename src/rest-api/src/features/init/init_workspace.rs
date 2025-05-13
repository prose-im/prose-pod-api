// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::{http::HeaderValue, Json};
use serde::{Deserialize, Serialize};
use service::{
    init::InitService, secrets::SecretsStore, server_config::ServerConfig, workspace::Workspace,
    xmpp::XmppServiceInner, AppConfig,
};

use crate::{error::prelude::*, features::workspace_details::WORKSPACE_ROUTE, responders::Created};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitWorkspaceRequest {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

pub async fn init_workspace_route(
    init_service: InitService,
    app_config: AppConfig,
    secrets_store: SecretsStore,
    xmpp_service: XmppServiceInner,
    server_config: ServerConfig,
    Json(req): Json<InitWorkspaceRequest>,
) -> Result<Created<Workspace>, Error> {
    let workspace = init_service
        .init_workspace(
            Arc::new(app_config),
            Arc::new(secrets_store),
            Arc::new(xmpp_service),
            &server_config,
            req.clone(),
        )
        .await?;

    let response = Workspace {
        name: req.name,
        accent_color: workspace.accent_color,
        icon: None,
    };

    let resource_uri = WORKSPACE_ROUTE;
    Ok(Created {
        location: HeaderValue::from_static(resource_uri),
        body: response,
    })
}

// BOILERPLATE

impl Into<Workspace> for InitWorkspaceRequest {
    fn into(self) -> Workspace {
        Workspace {
            name: self.name,
            accent_color: self.accent_color,
            icon: None,
        }
    }
}
