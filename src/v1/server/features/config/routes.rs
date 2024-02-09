// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{get, post, put};

// TODO: Routes to restore defaults

/// Get the current configuration of server features.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[get("/v1/server/features/config")]
pub(super) fn get_features_config() -> String {
    todo!()
}

/// Activate or deactivate message archiving.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/server/features/config/store-message-archive")]
pub(super) fn store_message_archive() -> String {
    todo!()
}

/// Update message archive retention time.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/server/features/config/message-archive-retention")]
pub(super) fn message_archive_retention() -> String {
    todo!()
}

/// Expunge the message archive.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[post("/v1/server/features/config/expunge-message-archive")]
pub(super) fn expunge_message_archive() -> String {
    todo!()
}

/// Activate or deactivate file upload and sharing.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/server/features/config/file-upload")]
pub(super) fn file_upload() -> String {
    todo!()
}

/// Change the file storage encryption scheme.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/server/features/config/file-storage-encryption-scheme")]
pub(super) fn file_storage_encryption_scheme() -> String {
    todo!()
}

/// Change the retention time of uploaded files.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[put("/v1/server/features/config/file-retention")]
pub(super) fn file_retention() -> String {
    todo!()
}

/// Expunge the file storage.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = String)
    )
)]
#[post("/v1/server/features/config/expunge-file-storage")]
pub(super) fn expunge_file_storage() -> String {
    todo!()
}
