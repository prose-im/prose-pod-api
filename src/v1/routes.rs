// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::JID;
use entity::server_config;
use rocket::serde::json::Json;
use rocket::State;
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::sea_orm::{Set, TransactionTrait as _, TryIntoModel};
use service::{Mutation, Query};
use utoipa::openapi::PathItemType::Put;
use utoipa::OpenApi;
use utoipauto::utoipauto;

use crate::guards::{Db, JWTService, UserFactory};

use super::workspace::openapi_extensions;
use crate::error::Error;
use crate::v1::workspace::rocket_uri_macro_set_workspace_icon_file;

pub type R<T> = Result<Json<T>, Error>;

#[utoipauto(paths = "src/v1")]
#[derive(OpenApi)]
#[openapi()]
pub struct ApiDoc;

#[get("/v1/api-docs/openapi.json")]
/// Construct the OpenAPI description file for this version of the API.
pub(super) fn openapi() -> String {
    let mut open_api = ApiDoc::openapi();

    // Crate `utoipa` doesn't support request bodies with multiple content types,
    // we need to override the definition manually.
    open_api
        .paths
        .paths
        .get_mut(&uri!(set_workspace_icon_file).to_string())
        .unwrap()
        .operations
        .insert(Put, openapi_extensions::set_workspace_icon());

    open_api.to_pretty_json().unwrap()
}

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
    let txn = db.begin().await.map_err(Error::DbErr)?;

    let server_config = Query::server_config(db).await.map_err(Error::DbErr)?;
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
        .await
        // TODO: Log as "Could not create server config"
        .map_err(Error::DbErr)?
        .try_into_model()
        // TODO: Log as "Could not transform active model into model"
        .map_err(Error::DbErr)?;

    user_factory
        .create_user(&req.admin.jid, &req.admin.password, &req.admin.nickname)
        .await?;

    // Commit the transaction only if the admin user was
    // successfully created, to prevent inconsistent states.
    txn.commit().await.map_err(Error::DbErr)?;

    Ok(Json(server_config))
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub jid: JID,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

/// Log user in and return an authentication token.
#[post("/v1/login", format = "json", data = "<req>")]
pub(super) fn login(
    // conn: Connection<'_, Db>,
    jwt_service: &State<JWTService>,
    req: Json<LoginRequest>,
) -> R<LoginResponse> {
    // FIXME: Add password authentication, this is unsecure!

    let token = jwt_service.generate_jwt(&req.jid)?;

    let response = LoginResponse { token }.into();

    Ok(response)
}
