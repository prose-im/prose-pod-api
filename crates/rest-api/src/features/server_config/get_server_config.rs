// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, serde::json::Json};
use service::server_config::ServerConfig;

use crate::{error::Error, guards::LazyGuard};

#[get("/v1/server/config")]
pub async fn get_server_config_route(
    server_config: LazyGuard<ServerConfig>,
) -> Result<Json<ServerConfig>, Error> {
    let model = server_config.inner?;
    Ok(model.into())
}
