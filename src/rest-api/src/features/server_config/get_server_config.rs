// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, http::header::IF_NONE_MATCH, response::NoContent, Json};
use axum_extra::{headers::IfNoneMatch, TypedHeader};
use service::server_config::{ServerConfig, ServerConfigRepository};

use crate::{
    error::{Error, PreconditionRequired},
    features::init::ServerConfigNotInitialized,
    AppState,
};

pub async fn get_server_config_route(server_config: ServerConfig) -> Json<ServerConfig> {
    Json(server_config)
}

pub async fn is_server_initialized_route(
    State(AppState { db, .. }): State<AppState>,
    TypedHeader(if_none_match): TypedHeader<IfNoneMatch>,
) -> Result<NoContent, Error> {
    if if_none_match != IfNoneMatch::any() {
        Err(Error::from(PreconditionRequired {
            comment: format!("Missing header: '{IF_NONE_MATCH}'."),
        }))
    } else if ServerConfigRepository::is_initialized(&db).await? {
        Ok(NoContent)
    } else {
        Err(Error::from(ServerConfigNotInitialized))
    }
}
