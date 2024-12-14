// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod file_upload;
mod get_server_config;
mod message_archive;
mod push_notifications;
mod util;

use axum::routing::{get, put};

pub use file_upload::*;
pub use get_server_config::*;
pub use message_archive::*;
pub use push_notifications::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        // Server config
        get_server_config_route,
        // File upload
        reset_files_config_route,
        set_file_upload_allowed_route,
        set_file_storage_encryption_scheme_route,
        set_file_storage_retention_route,
        // Message archive
        reset_messaging_config_route,
        set_message_archive_enabled_route,
        set_message_archive_retention_route,
        reset_message_archive_retention_route,
        // Push notifications
        reset_push_notifications_config_route,
        set_push_notification_with_body_route,
        reset_push_notification_with_body_route,
        set_push_notification_with_sender_route,
        reset_push_notification_with_sender_route,
    ]
}

pub(super) fn router<S: crate::AxumState>() -> axum::Router<S> {
    axum::Router::new()
        // Server config
        .route("/v1/server/config", get(get_server_config_route_axum))
        // File upload
        .route(
            "/v1/server/config/files/reset",
            put(reset_files_config_route_axum),
        )
        .route(
            "/v1/server/config/file-upload-allowed",
            put(set_file_upload_allowed_route_axum),
        )
        .route(
            "/v1/server/config/file-storage-encryption-scheme",
            put(set_file_storage_encryption_scheme_route_axum),
        )
        .route(
            "/v1/server/config/file-storage-retention",
            put(set_file_storage_retention_route_axum),
        )
        // Message archive
        .route(
            "/v1/server/config/messaging/reset",
            put(reset_messaging_config_route_axum),
        )
        .route(
            "/v1/server/config/message-archive-enabled",
            put(set_message_archive_enabled_route_axum),
        )
        .route(
            "/v1/server/config/message-archive-retention",
            put(set_message_archive_retention_route_axum),
        )
        .route(
            "/v1/server/config/message-archive-retention/reset",
            put(reset_message_archive_retention_route_axum),
        )
        // Push notifications
        .route(
            "/v1/server/config/push-notifications/reset",
            put(reset_push_notifications_config_route_axum),
        )
        .route(
            "/v1/server/config/push-notification-with-body",
            put(set_push_notification_with_body_route_axum),
        )
        .route(
            "/v1/server/config/push-notification-with-body/reset",
            put(reset_push_notification_with_body_route_axum),
        )
        .route(
            "/v1/server/config/push-notification-with-sender",
            put(set_push_notification_with_sender_route_axum),
        )
        .route(
            "/v1/server/config/push-notification-with-sender/reset",
            put(reset_push_notification_with_sender_route_axum),
        )
}
