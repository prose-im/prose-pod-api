// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod guards;
mod prosody_routes;
mod routes;

use axum::{
    middleware::from_extractor_with_state,
    routing::{delete, get, put, MethodRouter},
};

use crate::AppState;

use super::{auth::guards::IsAdmin, init::SERVER_CONFIG_ROUTE};

use self::prosody_routes::*;
use self::routes::*;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            SERVER_CONFIG_ROUTE,
            axum::Router::new()
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
                // Prosody
                .route("/prosody", get(get_prosody_config_lua_route))
                .route(
                    "/prosody-overrides",
                    MethodRouter::new()
                        .put(set_prosody_overrides_route)
                        .get(get_prosody_overrides_route)
                        .delete(delete_prosody_overrides_route),
                )
                .route(
                    "/prosody-overrides-raw",
                    MethodRouter::new()
                        .put(set_prosody_overrides_raw_route)
                        .get(get_prosody_overrides_raw_route)
                        .delete(delete_prosody_overrides_raw_route),
                )
                // Require authentication
                .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone())),
        )
        .route(
            SERVER_CONFIG_ROUTE,
            MethodRouter::new()
                // NOTE: `GET /v1/server/config` handles authentication itself,
                //   so it can return only the domain when called
                //   unauthenticated (requirement of the Dashboard).
                .get(get_server_config_route)
                // NOTE: `HEAD /v1/server/config` doesn’t require authentication.
                .head(is_server_initialized_route),
        )
        .with_state(app_state)
}
