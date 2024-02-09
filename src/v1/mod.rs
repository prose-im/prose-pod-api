// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod error;
mod members;
mod server;
pub mod workspace;

use entity::settings;
use migration::sea_orm::{Set, TryIntoModel};
use rocket::routes;
use rocket::serde::json::Json;
use rocket::Route;
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use service::{Mutation, Query};
use utoipa::openapi::PathItemType::Put;
use utoipa::OpenApi;
use utoipauto::utoipauto;

use crate::pool::Db;

use self::error::Error;
use self::workspace::openapi_extensions;
use crate::v1::workspace::rocket_uri_macro_set_workspace_icon_file;

pub(super) fn routes() -> Vec<Route> {
    vec![
        routes![openapi, init],
        members::routes(),
        server::routes(),
        workspace::routes(),
    ]
    .concat()
}

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

pub type InitResponse = settings::Model;

/// Initialize the Prose Pod and return the default configuration.
#[post("/v1/init", format = "json", data = "<req>")]
pub(super) async fn init(
    conn: Connection<'_, Db>,
    req: Json<InitRequest>,
) -> R<InitResponse> {
    let db = conn.into_inner();

    let settings = Query::settings(db).await.map_err(Error::DbErr)?;
    let None = settings else {
        return Err(Error::PodAlreadyInitialized);
    };

    let req = req.into_inner();
    let form = settings::ActiveModel {
        workspace_name: Set(req.workspace_name),
        ..Default::default()
    };
    let settings = Mutation::create_settings(db, form)
        .await
        .expect("Could not create settings")
        .try_into_model()
        .expect("Could not transform active model into model");
    Ok(Json(settings))
}
