// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{response::status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use service::{
    config::AppConfig,
    controllers::init_controller::{InitController, InitFirstAccountForm},
    model::{JidNode, ServerConfig, Workspace},
    repositories::{ServerConfigCreateForm, WorkspaceCreateForm},
    services::server_ctl::ServerCtl,
};

use crate::{
    forms::JID as JIDUriParam,
    guards::{LazyGuard, UnauthenticatedUserService},
    model::SerializableSecretString,
    v1::members::{rocket_uri_macro_get_member, Member},
    v1::Created,
};

#[derive(Serialize, Deserialize)]
pub struct InitWorkspaceRequest {
    /// Organization name.
    pub name: String,
    /// Color used in the Prose workspace, as a HEX color (e.g. `#1972F5`).
    pub accent_color: Option<String>,
}

impl Into<WorkspaceCreateForm> for InitWorkspaceRequest {
    fn into(self) -> WorkspaceCreateForm {
        WorkspaceCreateForm {
            name: self.name,
            accent_color: Some(self.accent_color),
        }
    }
}

#[put("/v1/workspace", format = "json", data = "<req>")]
pub async fn init_workspace<'r>(
    init_controller: LazyGuard<InitController<'r>>,
    req: Json<InitWorkspaceRequest>,
) -> Created<Workspace> {
    let init_controller = init_controller.inner?;

    let workspace = init_controller.init_workspace(req.into_inner()).await?;

    let resource_uri = uri!(crate::v1::workspace::get_workspace).to_string();
    Ok(status::Created::new(resource_uri).body(workspace.into()))
}

#[derive(Serialize, Deserialize)]
pub struct InitServerConfigRequest {
    /// XMPP server domain (e.g. `crisp.chat`).
    /// This is what will appear in JIDs (e.g. `valerian@`**`crisp.chat`**).
    pub domain: String,
}

impl Into<ServerConfigCreateForm> for InitServerConfigRequest {
    fn into(self) -> ServerConfigCreateForm {
        ServerConfigCreateForm {
            domain: self.domain.to_owned(),
        }
    }
}

#[put("/v1/server/config", format = "json", data = "<req>")]
pub async fn init_server_config<'r>(
    init_controller: LazyGuard<InitController<'r>>,
    server_ctl: &State<ServerCtl>,
    app_config: &State<AppConfig>,
    req: Json<InitServerConfigRequest>,
) -> Created<ServerConfig> {
    let init_controller = init_controller.inner?;
    let form = req.into_inner();

    let server_config = init_controller
        .init_server_config(server_ctl, app_config, form)
        .await?;

    let resource_uri = uri!(crate::v1::server::config::get_server_config).to_string();
    Ok(status::Created::new(resource_uri).body(server_config.into()))
}

#[derive(Serialize, Deserialize)]
pub struct InitFirstAccountRequest {
    pub username: JidNode,
    pub password: SerializableSecretString,
    pub nickname: String,
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

#[put("/v1/init/first-account", format = "json", data = "<req>")]
pub async fn init_first_account<'r>(
    init_controller: LazyGuard<InitController<'r>>,
    server_config: LazyGuard<ServerConfig>,
    user_service: LazyGuard<UnauthenticatedUserService<'r>>,
    req: Json<InitFirstAccountRequest>,
) -> Created<Member> {
    let init_controller = init_controller.inner?;
    let server_config = &server_config.inner?;
    let user_service = &user_service.inner?;
    let form = req.into_inner();

    let member = init_controller
        .init_first_account(server_config, user_service, form)
        .await?;

    let resource_uri = uri!(get_member(member.jid())).to_string();
    let response = Member::from(member);
    Ok(status::Created::new(resource_uri).body(response.into()))
}
