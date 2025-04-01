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
    routing::{delete, get, head, put, MethodRouter},
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
                .route("/files", delete(reset_files_config_route))
                .route(
                    "/file-upload-allowed",
                    MethodRouter::new()
                        .put(set_file_upload_allowed_route)
                        .get(get_file_upload_allowed_route)
                        .delete(reset_file_upload_allowed_route),
                )
                .route(
                    "/file-storage-encryption-scheme",
                    put(set_file_storage_encryption_scheme_route),
                )
                .route(
                    "/file-storage-retention",
                    MethodRouter::new()
                        .put(set_file_storage_retention_route)
                        .get(get_file_storage_retention_route)
                        .delete(reset_file_storage_retention_route),
                )
                // Message archive
                .route("/messaging", delete(reset_messaging_config_route))
                .route(
                    "/message-archive-enabled",
                    MethodRouter::new()
                        .put(set_message_archive_enabled_route)
                        .get(get_message_archive_enabled_route)
                        .delete(reset_message_archive_enabled_route),
                )
                .route(
                    "/message-archive-retention",
                    MethodRouter::new()
                        .put(set_message_archive_retention_route)
                        .get(get_message_archive_retention_route)
                        .delete(reset_message_archive_retention_route),
                )
                // Push notifications
                .route(
                    "/push-notifications",
                    delete(reset_push_notifications_config_route),
                )
                .route(
                    "/push-notification-with-body",
                    MethodRouter::new()
                        .put(set_push_notification_with_body_route)
                        .get(get_push_notification_with_body_route)
                        .delete(reset_push_notification_with_body_route),
                )
                .route(
                    "/push-notification-with-sender",
                    MethodRouter::new()
                        .put(set_push_notification_with_sender_route)
                        .get(get_push_notification_with_sender_route)
                        .delete(reset_push_notification_with_sender_route),
                )
                // Network encryption
                .route(
                    "/network-encryption",
                    delete(reset_network_encryption_config_route),
                )
                .route(
                    "/tls-profile",
                    MethodRouter::new()
                        .put(set_tls_profile_route)
                        .get(get_tls_profile_route)
                        .delete(reset_tls_profile_route),
                )
                // Server federation
                .route(
                    "/server-federation",
                    delete(reset_server_federation_config_route),
                )
                .route(
                    "/federation-enabled",
                    MethodRouter::new()
                        .put(set_federation_enabled_route)
                        .get(get_federation_enabled_route)
                        .delete(reset_federation_enabled_route),
                )
                .route(
                    "/federation-whitelist-enabled",
                    MethodRouter::new()
                        .put(set_federation_whitelist_enabled_route)
                        .get(get_federation_whitelist_enabled_route)
                        .delete(reset_federation_whitelist_enabled_route),
                )
                .route(
                    "/federation-friendly-servers",
                    MethodRouter::new()
                        .put(set_federation_friendly_servers_route)
                        .get(get_federation_friendly_servers_route)
                        .delete(reset_federation_friendly_servers_route),
                )
                // Require authentication
                .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone())),
        )
        // NOTE: `HEAD /v1/server/config` doesn’t require authentication.
        .route(SERVER_CONFIG_ROUTE, head(is_server_initialized_route))
        .with_state(app_state)
}
