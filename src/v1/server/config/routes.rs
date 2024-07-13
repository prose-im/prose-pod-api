// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use service::{
    model::{DateLike, Duration, PossiblyInfinite, ServerConfig},
    services::server_manager::ServerManager,
};

use crate::{error, guards::LazyGuard, v1::R};

#[get("/v1/server/config")]
pub(super) async fn get_server_config(server_config: LazyGuard<ServerConfig>) -> R<ServerConfig> {
    let model = server_config.inner?;
    Ok(model.into())
}

/// Generates a route for setting a specific server config.
/// Aslo generates its associated request type.
macro_rules! set_route {
    ($route:expr, $req_type:ident, $var_type:ty, $var:ident, $fn:ident) => {
        #[derive(Serialize, Deserialize)]
        pub struct $req_type {
            pub $var: $var_type,
        }

        #[put($route, format = "json", data = "<req>")]
        pub(super) async fn $fn(
            server_manager: LazyGuard<ServerManager<'_>>,
            req: Json<$req_type>,
        ) -> R<ServerConfig> {
            let server_manager = server_manager.inner?;
            let new_state: $var_type = req.$var.to_owned();
            let new_config = server_manager.$fn(new_state).await?;
            Ok(new_config.into())
        }
    };
}
/// Generates a route for resetting a specific server config.
macro_rules! reset_route {
    ($route:expr, $fn:ident) => {
        #[put($route)]
        pub(super) async fn $fn(server_manager: LazyGuard<ServerManager<'_>>) -> R<ServerConfig> {
            let server_manager = server_manager.inner?;
            let new_config = server_manager.$fn().await?;
            Ok(new_config.into())
        }
    };
}

reset_route!("/v1/server/config/messaging/reset", reset_messaging_config);

set_route!(
    "/v1/server/config/store-message-archive",
    SetMessageArchiveEnabledRequest,
    bool,
    message_archive_enabled,
    set_message_archive_enabled
);

set_route!(
    "/v1/server/config/message-archive-retention",
    SetMessageArchiveRetentionRequest,
    PossiblyInfinite<Duration<DateLike>>,
    message_archive_retention,
    set_message_archive_retention
);
reset_route!(
    "/v1/server/config/message-archive-retention/reset",
    reset_message_archive_retention
);

reset_route!("/v1/server/config/files/reset", reset_files_config);

set_route!(
    "/v1/server/config/allow-file-upload",
    SetFileUploadAllowedRequest,
    bool,
    file_upload_allowed,
    set_file_upload_allowed
);

#[put("/v1/server/config/file-storage-encryption-scheme")]
pub(super) fn set_file_storage_encryption_scheme() -> R<ServerConfig> {
    Err(error::NotImplemented("File storage encryption scheme").into())
}

set_route!(
    "/v1/server/config/file-storage-retention",
    SetFileStorageRetentionRequest,
    PossiblyInfinite<Duration<DateLike>>,
    file_storage_retention,
    set_file_storage_retention
);
