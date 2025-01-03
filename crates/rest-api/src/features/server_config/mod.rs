// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
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

use super::init::SERVER_CONFIG_ROUTE;

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        // Server config
        .route(SERVER_CONFIG_ROUTE, get(get_server_config_route))
        // File upload
        .route(
            "/v1/server/config/files/reset",
            put(reset_files_config_route),
        )
        .route(
            "/v1/server/config/file-upload-allowed",
            put(set_file_upload_allowed_route),
        )
        .route(
            "/v1/server/config/file-storage-encryption-scheme",
            put(set_file_storage_encryption_scheme_route),
        )
        .route(
            "/v1/server/config/file-storage-retention",
            put(set_file_storage_retention_route),
        )
        // Message archive
        .route(
            "/v1/server/config/messaging/reset",
            put(reset_messaging_config_route),
        )
        .route(
            "/v1/server/config/message-archive-enabled",
            put(set_message_archive_enabled_route),
        )
        .route(
            "/v1/server/config/message-archive-retention",
            put(set_message_archive_retention_route),
        )
        .route(
            "/v1/server/config/message-archive-retention/reset",
            put(reset_message_archive_retention_route),
        )
        // Push notifications
        .route(
            "/v1/server/config/push-notifications/reset",
            put(reset_push_notifications_config_route),
        )
        .route(
            "/v1/server/config/push-notification-with-body",
            put(set_push_notification_with_body_route),
        )
        .route(
            "/v1/server/config/push-notification-with-body/reset",
            put(reset_push_notification_with_body_route),
        )
        .route(
            "/v1/server/config/push-notification-with-sender",
            put(set_push_notification_with_sender_route),
        )
        .route(
            "/v1/server/config/push-notification-with-sender/reset",
            put(reset_push_notification_with_sender_route),
        )
}
