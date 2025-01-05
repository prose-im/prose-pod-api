// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use service::server_config::ServerConfig;

use crate::error::Error;

pub async fn get_server_config_route(
    server_config: ServerConfig,
) -> Result<Json<ServerConfig>, Error> {
    Ok(Json(server_config))
}
