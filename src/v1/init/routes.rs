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
use service::repositories::{
    MemberRepository, ServerConfig, ServerConfigCreateForm, Workspace, WorkspaceCreateForm,
    WorkspaceRepository,
};
use service::sea_orm::TransactionTrait as _;
use service::ServerCtl;
use service::{JIDNode, MemberRole};

use crate::error::Error;
use crate::forms::JID as JIDUriParam;
use crate::guards::{
    self, Db, LazyGuard, UnauthenticatedServerManager, UnauthenticatedUserFactory,
};
use crate::util::bare_jid_from_username;
use crate::v1::members::{rocket_uri_macro_get_member, Member};
use crate::v1::Created;

#[derive(Default, Serialize, Deserialize)]
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

/// Initialize the Prose Pod and return the default configuration.
#[put("/v1/workspace", format = "json", data = "<req>")]
pub async fn init_workspace(
    conn: Connection<'_, Db>,
    req: Json<InitWorkspaceRequest>,
) -> Created<Workspace> {
    let db = conn.into_inner();

    // Check that the workspace isn't already initialized.
    let None = WorkspaceRepository::get(db).await? else {
        return Err(Error::WorkspaceAlreadyInitialized);
    };

    let workspace = WorkspaceRepository::create(db, req.into_inner()).await?;

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

/// Initialize the Prose Pod and return the default configuration.
#[put("/v1/server/config", format = "json", data = "<req>")]
pub async fn init_server_config(
    conn: Connection<'_, Db>,
    server_ctl: &State<ServerCtl>,
    app_config: &State<AppConfig>,
    req: Json<InitServerConfigRequest>,
) -> Created<ServerConfig> {
    let db = conn.into_inner();
    let req = req.into_inner();

    let server_config =
        UnauthenticatedServerManager::init_server_config(db, server_ctl, app_config, req).await?;

    let resource_uri = uri!(crate::v1::server::config::get_server_config).to_string();
    Ok(status::Created::new(resource_uri).body(server_config.into()))
}

#[derive(Serialize, Deserialize)]
pub struct InitFirstAccountRequest {
    /// JID node (e.g. `valerian` in `valerian@crisp.chat`).
    pub username: JIDNode,
    /// As the name suggests, a password.
    pub password: String,
    /// vCard NICKNAME (i.e. what will be displayed to users).
    pub nickname: String,
}

/// Initialize the Prose Pod and return the default configuration.
#[put("/v1/init/first-account", format = "json", data = "<req>")]
pub async fn init_first_account(
    conn: Connection<'_, Db>,
    server_config: LazyGuard<guards::ServerConfig>,
    user_factory: LazyGuard<UnauthenticatedUserFactory<'_>>,
    req: Json<InitFirstAccountRequest>,
) -> Created<Member> {
    let db = conn.into_inner();

    if MemberRepository::count(db).await? > 0 {
        return Err(Error::FirstAccountAlreadyCreated);
    }

    let server_config = server_config.inner?;
    let user_factory = user_factory.inner?;

    let jid = bare_jid_from_username(req.username.to_string(), &server_config)?;
    let txn = db.begin().await?;
    let member = user_factory
        .create_user(
            &txn,
            &jid,
            &req.password,
            &req.nickname,
            &Some(MemberRole::Admin),
        )
        .await?;
    txn.commit().await?;

    let resource_uri = uri!(get_member(jid)).to_string();
    let response = Member::from(member);
    Ok(status::Created::new(resource_uri).body(response.into()))
}
