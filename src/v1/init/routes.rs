// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::config::Config as AppConfig;
use service::controllers::init_controller::{InitController, InitFirstAccountForm};
use service::model::{JIDNode, ServerConfig, Workspace};
use service::repositories::{ServerConfigCreateForm, WorkspaceCreateForm};
use service::services::server_ctl::ServerCtl;

use crate::forms::JID as JIDUriParam;
use crate::guards::{Db, LazyGuard, UnauthenticatedUserService};
use crate::model::SerializableSecretString;
use crate::v1::members::{rocket_uri_macro_get_member, Member};
use crate::v1::Created;

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
pub async fn init_workspace(
    conn: Connection<'_, Db>,
    req: Json<InitWorkspaceRequest>,
) -> Created<Workspace> {
    let db = conn.into_inner();
    let form = req.into_inner();

    let workspace = InitController::init_workspace(db, form).await?;

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
pub async fn init_server_config(
    conn: Connection<'_, Db>,
    server_ctl: &State<ServerCtl>,
    app_config: &State<AppConfig>,
    req: Json<InitServerConfigRequest>,
) -> Created<ServerConfig> {
    let db = conn.into_inner();
    let req = req.into_inner();

    let server_config = InitController::init_server_config(db, server_ctl, app_config, req).await?;

    let resource_uri = uri!(crate::v1::server::config::get_server_config).to_string();
    Ok(status::Created::new(resource_uri).body(server_config.into()))
}

#[derive(Serialize, Deserialize)]
pub struct InitFirstAccountRequest {
    pub username: JIDNode,
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
pub async fn init_first_account(
    conn: Connection<'_, Db>,
    server_config: LazyGuard<ServerConfig>,
    user_service: LazyGuard<UnauthenticatedUserService<'_>>,
    req: Json<InitFirstAccountRequest>,
) -> Created<Member> {
    let db = conn.into_inner();
    let server_config = server_config.inner?;
    let user_service = user_service.inner?;
    let form = req.into_inner();

    let member =
        InitController::init_first_account(db, &server_config, &user_service, form).await?;

    let resource_uri = uri!(get_member(member.jid())).to_string();
    let response = Member::from(member);
    Ok(status::Created::new(resource_uri).body(response.into()))
}
