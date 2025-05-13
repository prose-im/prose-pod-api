// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use axum::{extract::State, http::HeaderValue, Json};
use service::{
    init::{InitFirstAccountForm, InitService},
    members::{Member, MemberRepository, UnauthenticatedMemberService},
    models::SerializableSecretString,
    secrets::SecretsStore,
    server_config::server_config_controller,
    workspace::Workspace,
    xmpp::{JidNode, XmppServiceInner},
    AppConfig,
};

use crate::{
    error::prelude::*, features::workspace_details::WORKSPACE_ROUTE, responders::Created, AppState,
};

use super::errors::ServerConfigNotInitialized;

// MARK: INIT WORKSPACE

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InitWorkspaceRequest {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

pub async fn init_workspace_route(
    State(AppState { db, .. }): State<AppState>,
    init_service: InitService,
    app_config: AppConfig,
    secrets_store: SecretsStore,
    xmpp_service: XmppServiceInner,
    Json(req): Json<InitWorkspaceRequest>,
) -> Result<Created<Workspace>, Error> {
    let server_domain = (server_config_controller::get_server_domain(&db).await)?
        .ok_or(ServerConfigNotInitialized)?;

    let workspace = init_service
        .init_workspace(
            Arc::new(app_config),
            Arc::new(secrets_store),
            Arc::new(xmpp_service),
            &server_domain,
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

// MARK: INIT FIRST ACCOUNT

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InitFirstAccountRequest {
    pub username: JidNode,
    pub password: SerializableSecretString,
    pub nickname: String,
}

pub async fn init_first_account_route(
    State(AppState { ref db, .. }): State<AppState>,
    init_service: InitService,
    member_service: UnauthenticatedMemberService,
    Json(req): Json<InitFirstAccountRequest>,
) -> Result<Created<Member>, Error> {
    let server_domain = (server_config_controller::get_server_domain(db).await)?
        .ok_or(ServerConfigNotInitialized)?;

    let member = init_service
        .init_first_account(&server_domain, &member_service, req)
        .await?;

    let resource_uri = format!("/v1/members/{jid}", jid = member.jid);
    Ok(Created {
        location: HeaderValue::from_str(&resource_uri)?,
        body: Member::from(member),
    })
}

pub async fn is_first_account_created_route(
    State(AppState { db, .. }): State<AppState>,
) -> StatusCode {
    if MemberRepository::count(&db).await.unwrap_or_default() == 0 {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::OK
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

impl Into<InitFirstAccountForm> for InitFirstAccountRequest {
    fn into(self) -> InitFirstAccountForm {
        InitFirstAccountForm {
            username: self.username,
            password: self.password.into(),
            nickname: self.nickname,
        }
    }
}
