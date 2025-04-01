// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::Json;
use service::{
    models::durations::{DateLike, Duration, PossiblyInfinite},
    server_config::ServerConfig,
};

use crate::{
    error::{self, Error},
    server_config_reset_route, server_config_routes,
};

server_config_reset_route!(reset_files_config, reset_files_config_route);

server_config_routes!(
            key: file_upload_allowed, type: bool,
      set:   set_file_upload_allowed_route using   set_file_upload_allowed,
      get:   get_file_upload_allowed_route,
    reset: reset_file_upload_allowed_route using reset_file_upload_allowed,
);

pub async fn set_file_storage_encryption_scheme_route() -> Result<Json<ServerConfig>, Error> {
    Err(Error::from(error::NotImplemented(
        "File storage encryption scheme",
    )))
}

server_config_routes!(
            key: file_storage_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_file_storage_retention_route using   set_file_storage_retention,
      get:   get_file_storage_retention_route,
    reset: reset_file_storage_retention_route using reset_file_storage_retention,
);
