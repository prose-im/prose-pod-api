// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod file_upload;
mod get_server_config;
mod message_archive;
mod network_encryption;
mod push_notifications;
mod server_federation;
mod util;

use axum::{
    middleware::from_extractor_with_state,
    routing::{get, put},
};

pub use file_upload::*;
pub use get_server_config::*;
pub use message_archive::*;
pub use network_encryption::*;
pub use push_notifications::*;
pub use server_federation::*;

use crate::AppState;

use super::{auth::guards::IsAdmin, init::SERVER_CONFIG_ROUTE};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            SERVER_CONFIG_ROUTE,
            axum::Router::new()
                // Server config
                .route("/", get(get_server_config_route))
                // File upload
                .route("/files/reset", put(reset_files_config_route))
                .route("/file-upload-allowed", put(set_file_upload_allowed_route))
                .route(
                    "/file-storage-encryption-scheme",
                    put(set_file_storage_encryption_scheme_route),
                )
                .route(
                    "/file-storage-retention",
                    put(set_file_storage_retention_route),
                )
                // Message archive
                .route("/messaging/reset", put(reset_messaging_config_route))
                .route(
                    "/message-archive-enabled",
                    put(set_message_archive_enabled_route),
                )
                .route(
                    "/message-archive-retention",
                    put(set_message_archive_retention_route),
                )
                .route(
                    "/message-archive-retention/reset",
                    put(reset_message_archive_retention_route),
                )
                // Push notifications
                .route(
                    "/push-notifications/reset",
                    put(reset_push_notifications_config_route),
                )
                .route(
                    "/push-notification-with-body",
                    put(set_push_notification_with_body_route),
                )
                .route(
                    "/push-notification-with-body/reset",
                    put(reset_push_notification_with_body_route),
                )
                .route(
                    "/push-notification-with-sender",
                    put(set_push_notification_with_sender_route),
                )
                .route(
                    "/push-notification-with-sender/reset",
                    put(reset_push_notification_with_sender_route),
                )
                // Network encryption
                .route(
                    "/network-encryption/reset",
                    put(reset_network_encryption_config_route),
                )
                .route("/tls-profile", put(set_tls_profile_route))
                .route("/tls-profile/reset", put(reset_tls_profile_route))
                // Server federation
                .route(
                    "/server-federation/reset",
                    put(reset_server_federation_config_route),
                )
                .route(
                    "/federation-enabled/reset",
                    put(reset_federation_enabled_route),
                )
                .route("/federation-enabled", put(set_federation_enabled_route))
                .route(
                    "/federation-whitelist-enabled/reset",
                    put(reset_federation_whitelist_enabled_route),
                )
                .route(
                    "/federation-whitelist-enabled",
                    put(set_federation_whitelist_enabled_route),
                )
                .route(
                    "/federation-friendly-servers/reset",
                    put(reset_federation_friendly_servers_route),
                )
                .route(
                    "/federation-friendly-servers",
                    put(set_federation_friendly_servers_route),
                ),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}
