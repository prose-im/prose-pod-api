// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::{MemberRole, JID};
use entity::server_config;
use rocket::serde::json::Json;
use rocket::State;
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::sea_orm::{Set, TransactionTrait as _, TryIntoModel};
use service::{Mutation, Query, ServerCtl};

use crate::guards::{BasicAuth, Db, JWTService, UserFactory};

use crate::error::Error;

pub type R<T> = Result<Json<T>, Error>;

#[derive(Serialize, Deserialize)]
pub struct InitRequest {
    pub workspace_name: String,
    pub admin: AdminAccountInit,
}

#[derive(Serialize, Deserialize)]
pub struct AdminAccountInit {
    pub jid: JID,
    pub password: String,
    pub nickname: String,
}

pub type InitResponse = server_config::Model;

/// Initialize the Prose Pod and return the default configuration.
#[post("/v1/init", format = "json", data = "<req>")]
pub(super) async fn init(
    conn: Connection<'_, Db>,
    user_factory: UserFactory<'_>,
    req: Json<InitRequest>,
) -> R<InitResponse> {
    let db = conn.into_inner();
    let txn = db.begin().await?;

    let server_config = Query::server_config(db).await?;
    let None = server_config else {
        return Err(Error::PodAlreadyInitialized);
    };

    let req = req.into_inner();
    let form = server_config::ActiveModel {
        workspace_name: Set(req.workspace_name),
        ..Default::default()
    };
    // Initialize the server config in a transaction,
    // to rollback if subsequent operations fail.
    let server_config = Mutation::create_server_config(&txn, form)
        .await?
        .try_into_model()?;

    // NOTE: We can't rollback changes made to the XMPP server so let's do it
    //   after "rollbackable" DB changes in case they fail. It's not perfect
    //   but better than nothing.
    // TODO: Find a way to rollback XMPP server changes.
    user_factory
        .create_user(
            &txn,
            &req.admin.jid,
            &req.admin.password,
            &req.admin.nickname,
            &Some(MemberRole::Admin),
        )
        .await?;

    // Commit the transaction only if the admin user was
    // successfully created, to prevent inconsistent states.
    txn.commit().await?;

    Ok(Json(server_config))
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

/// Log user in and return an authentication token.
#[post("/v1/login")]
pub(super) fn login(
    basic_auth: BasicAuth,
    jwt_service: &State<JWTService>,
    server_ctl: &State<ServerCtl>,
) -> R<LoginResponse> {
    server_ctl
        .implem
        .lock()
        .expect("Serverctl lock poisonned")
        .test_user_password(&basic_auth.jid, &basic_auth.password)?;

    let token = jwt_service.generate_jwt(&basic_auth.jid)?;

    let response = LoginResponse { token }.into();

    Ok(response)
}
