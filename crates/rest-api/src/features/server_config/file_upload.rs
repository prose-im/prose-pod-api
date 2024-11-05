// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{put, serde::json::Json};
use service::{
    features::{server_config::ServerConfig, xmpp::ServerManager},
    model::{DateLike, Duration, PossiblyInfinite},
};

use crate::{
    error::{self, Error},
    guards::LazyGuard,
    server_config_reset_route, server_config_set_route,
};

server_config_reset_route!(
    "/v1/server/config/files/reset",
    reset_files_config,
    reset_files_config_route
);

server_config_set_route!(
    "/v1/server/config/file-upload-allowed",
    SetFileUploadAllowedRequest,
    bool,
    file_upload_allowed,
    set_file_upload_allowed,
    set_file_upload_allowed_route
);

#[put("/v1/server/config/file-storage-encryption-scheme")]
pub fn set_file_storage_encryption_scheme_route() -> Result<Json<ServerConfig>, Error> {
    Err(error::NotImplemented("File storage encryption scheme").into())
}

server_config_set_route!(
    "/v1/server/config/file-storage-retention",
    SetFileStorageRetentionRequest,
    PossiblyInfinite<Duration<DateLike>>,
    file_storage_retention,
    set_file_storage_retention,
    set_file_storage_retention_route
);
