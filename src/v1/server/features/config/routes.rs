// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::server_ctl::ServerCtl;
use crate::v1::ServerConfig;
use entity::server_config;
use migration::sea_orm::{ActiveModelTrait as _, Set};
use rocket::serde::json::Json;
use rocket::{get, post, put, State};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::pool::Db;
use crate::v1::error::Error;

pub type R<T> = Result<Json<T>, Error>;

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

#[derive(Serialize, Deserialize, ToSchema)]
#[cfg_attr(test, derive(Debug))]
pub struct SetMessageArchivingRequest {
    pub message_archive_enabled: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[cfg_attr(test, derive(Debug))]
pub struct SetMessageArchivingResponse {
    pub message_archive_enabled: bool,
}

/// Activate or deactivate message archiving.
#[utoipa::path(
    tag = "Server / Features / Configuration",
    responses(
        (status = 200, description = "Success", body = SetMessageArchivingRequest)
    )
)]
#[put("/v1/server/features/config/store-message-archive", format = "json", data = "<req>")]
pub(super) async fn store_message_archive(
    conn: Connection<'_, Db>,
    server_config: ServerConfig,
    server_ctl: &State<ServerCtl>,
    req: Json<SetMessageArchivingRequest>,
) -> R<SetMessageArchivingResponse> {
    let db = conn.into_inner();
    let server_config = server_config.model?;

    let new_state = req.message_archive_enabled.clone();

    let mut active: server_config::ActiveModel = server_config.into();
    active.message_archive_enabled = Set(new_state);
    let server_config = active.update(db).await.map_err(Error::DbErr)?;

    let response = SetMessageArchivingResponse {
        message_archive_enabled: server_config.message_archive_enabled,
    };

    let mut server_ctl = server_ctl.implem.lock().unwrap();
    if server_ctl.set_message_archiving(new_state) {
        server_ctl.save_config();
    }

    Ok(response.into())
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
