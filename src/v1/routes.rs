// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::BTreeMap;

use entity::model::JID;
use entity::server_config;
use jwt::SignWithKey as _;
use rocket::serde::json::Json;
use rocket::State;
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::sea_orm::{Set, TryIntoModel};
use service::{Mutation, Query};
use utoipa::openapi::PathItemType::Put;
use utoipa::OpenApi;
use utoipauto::utoipauto;

use crate::guards::{Db, JWTKey, JWT_JID_KEY};

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
}

pub type InitResponse = server_config::Model;

/// Initialize the Prose Pod and return the default configuration.
#[post("/v1/init", format = "json", data = "<req>")]
pub(super) async fn init(conn: Connection<'_, Db>, req: Json<InitRequest>) -> R<InitResponse> {
    let db = conn.into_inner();

    let server_config = Query::server_config(db).await.map_err(Error::DbErr)?;
    let None = server_config else {
        return Err(Error::PodAlreadyInitialized);
    };

    let req = req.into_inner();
    let form = server_config::ActiveModel {
        workspace_name: Set(req.workspace_name),
        ..Default::default()
    };
    let server_config = Mutation::create_server_config(db, form)
        .await
        // TODO: Log as "Could not create server config"
        .map_err(Error::DbErr)?
        .try_into_model()
        // TODO: Log as "Could not transform active model into model"
        .map_err(Error::DbErr)?;
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
    jwt_key: &State<JWTKey>,
    req: Json<LoginRequest>,
) -> R<LoginResponse> {
    // FIXME: Add password authentication, this is unsecure!

    let jwt_key = jwt_key.as_hmac_sha_256()?;
    let jid = &req.jid;

    let mut claims = BTreeMap::new();
    claims.insert(JWT_JID_KEY, jid.to_string());
    let token_str = claims
        .sign_with_key(&jwt_key)
        .map_err(|e| Error::InternalServerError {
            reason: format!("Could not sign JWT claims: {e}"),
        })?;

    let response = LoginResponse {
        token: token_str.to_string(),
    }
    .into();

    Ok(response)
}
