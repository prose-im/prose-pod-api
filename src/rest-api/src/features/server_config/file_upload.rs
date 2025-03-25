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
    server_config_reset_route, server_config_set_route,
};

server_config_reset_route!(reset_files_config, reset_files_config_route);

server_config_set_route!(
    SetFileUploadAllowedRequest,
    bool,
    file_upload_allowed,
    set_file_upload_allowed,
    set_file_upload_allowed_route
);

pub async fn set_file_storage_encryption_scheme_route() -> Result<Json<ServerConfig>, Error> {
    Err(Error::from(error::NotImplemented(
        "File storage encryption scheme",
    )))
}

server_config_set_route!(
    SetFileStorageRetentionRequest,
    PossiblyInfinite<Duration<DateLike>>,
    file_storage_retention,
    set_file_storage_retention,
    set_file_storage_retention_route
);
