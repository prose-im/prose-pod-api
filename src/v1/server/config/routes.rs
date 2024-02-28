// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model::{DateLike, Duration, PossiblyInfinite};
use entity::server_config::Model as ServerConfig;
use rocket::serde::json::Json;
use rocket::{get, put};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Error;
use crate::guards::{ServerConfig as ServerConfigGuard, ServerManager};

pub type R<T> = Result<Json<T>, Error>;

// TODO: Routes to restore defaults

/// Get the current configuration of server features.
#[utoipa::path(
    tag = "Server / Configuration",
    responses(
        (status = 200, description = "Success", body = ServerConfig),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[get("/v1/server/config")]
pub(super) async fn get_features_config(server_config: ServerConfigGuard) -> R<ServerConfig> {
    let model = server_config.model?;
    Ok(model.into())
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetMessageArchivingRequest {
    pub message_archive_enabled: bool,
}

/// Activate or deactivate message archiving.
#[utoipa::path(
    tag = "Server / Configuration",
    responses(
        (status = 200, description = "Success", body = ServerConfig),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[put(
    "/v1/server/config/store-message-archive",
    format = "json",
    data = "<req>"
)]
pub(super) async fn store_message_archive(
    server_manager: ServerManager<'_>,
    req: Json<SetMessageArchivingRequest>,
) -> R<ServerConfig> {
    let server_manager = server_manager.inner?;
    let new_state = req.message_archive_enabled.clone();
    let new_config = server_manager.set_message_archiving(new_state).await?;
    Ok(new_config.into())
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetMessageArchiveRetentionRequest {
    pub message_archive_retention: PossiblyInfinite<Duration<DateLike>>,
}

/// Update message archive retention.
#[utoipa::path(
    tag = "Server / Configuration",
    responses(
        (status = 200, description = "Success", body = ServerConfig),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[put(
    "/v1/server/config/message-archive-retention",
    format = "json",
    data = "<req>"
)]
pub(super) async fn message_archive_retention(
    server_manager: ServerManager<'_>,
    req: Json<SetMessageArchiveRetentionRequest>,
) -> R<ServerConfig> {
    let server_manager = server_manager.inner?;
    let new_state = req.message_archive_retention.clone();
    let new_config = server_manager
        .set_message_archive_retention(new_state)
        .await?;
    Ok(new_config.into())
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetFileUploadingRequest {
    pub file_upload_allowed: bool,
}

/// Activate or deactivate file upload and sharing.
#[utoipa::path(
    tag = "Server / Configuration",
    responses(
        (status = 200, description = "Success", body = ServerConfig),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[put("/v1/server/config/allow-file-upload", format = "json", data = "<req>")]
pub(super) async fn store_files(
    server_manager: ServerManager<'_>,
    req: Json<SetFileUploadingRequest>,
) -> R<ServerConfig> {
    let server_manager = server_manager.inner?;
    let new_state = req.file_upload_allowed.clone();
    let new_config = server_manager.set_file_uploading(new_state).await?;
    Ok(new_config.into())
}

/// Change the file storage encryption scheme.
#[utoipa::path(
    tag = "Server / Configuration",
    responses(
        (status = 200, description = "Success", body = ServerConfig),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[put("/v1/server/config/file-storage-encryption-scheme")]
pub(super) fn file_storage_encryption_scheme() -> R<ServerConfig> {
    Err(Error::NotImplemented {
        feature: "File storage encryption scheme".to_string(),
    })
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetFileRetentionRequest {
    pub file_retention: PossiblyInfinite<Duration<DateLike>>,
}

/// Change the retention of uploaded files.
#[utoipa::path(
    tag = "Server / Configuration",
    responses(
        (status = 200, description = "Success", body = ServerConfig),
        (status = 400, description = "Pod not initialized", body = Error),
        (status = 401, description = "Unauthorized", body = Error),
        (status = 409, description = "Pod already initialized", body = Error),
    )
)]
#[put("/v1/server/config/file-retention", format = "json", data = "<req>")]
pub(super) async fn file_retention(
    server_manager: ServerManager<'_>,
    req: Json<SetFileRetentionRequest>,
) -> R<ServerConfig> {
    let server_manager = server_manager.inner?;
    let new_state = req.file_retention.clone();
    let new_config = server_manager.set_file_retention(new_state).await?;
    Ok(new_config.into())
}
