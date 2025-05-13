// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod routes;

use axum::{
    middleware::from_extractor_with_state,
    routing::{delete, get, put, MethodRouter},
};
use service::auth::IsAdmin;

use crate::AppState;

use self::routes::*;

pub const SERVER_CONFIG_ROUTE: &'static str = "/v1/server/config";

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            SERVER_CONFIG_ROUTE,
            axum::Router::new()
                // File upload
                .route("/files", delete(reset_files_config))
                .route(
                    "/file-upload-allowed",
                    MethodRouter::new()
                        .put(set_file_upload_allowed)
                        .get(get_file_upload_allowed)
                        .delete(reset_file_upload_allowed),
                )
                .route(
                    "/file-storage-encryption-scheme",
                    put(set_file_storage_encryption_scheme),
                )
                .route(
                    "/file-storage-retention",
                    MethodRouter::new()
                        .put(set_file_storage_retention)
                        .get(get_file_storage_retention)
                        .delete(reset_file_storage_retention),
                )
                // Message archive
                .route("/messaging", delete(reset_messaging_config))
                .route(
                    "/message-archive-enabled",
                    MethodRouter::new()
                        .put(set_message_archive_enabled)
                        .get(get_message_archive_enabled)
                        .delete(reset_message_archive_enabled),
                )
                .route(
                    "/message-archive-retention",
                    MethodRouter::new()
                        .put(set_message_archive_retention)
                        .get(get_message_archive_retention)
                        .delete(reset_message_archive_retention),
                )
                // Push notifications
                .route(
                    "/push-notifications",
                    delete(reset_push_notifications_config),
                )
                .route(
                    "/push-notification-with-body",
                    MethodRouter::new()
                        .put(set_push_notification_with_body)
                        .get(get_push_notification_with_body)
                        .delete(reset_push_notification_with_body),
                )
                .route(
                    "/push-notification-with-sender",
                    MethodRouter::new()
                        .put(set_push_notification_with_sender)
                        .get(get_push_notification_with_sender)
                        .delete(reset_push_notification_with_sender),
                )
                // Network encryption
                .route(
                    "/network-encryption",
                    delete(reset_network_encryption_config),
                )
                .route(
                    "/tls-profile",
                    MethodRouter::new()
                        .put(set_tls_profile)
                        .get(get_tls_profile)
                        .delete(reset_tls_profile),
                )
                // Server federation
                .route("/server-federation", delete(reset_server_federation_config))
                .route(
                    "/federation-enabled",
                    MethodRouter::new()
                        .put(set_federation_enabled)
                        .get(get_federation_enabled)
                        .delete(reset_federation_enabled),
                )
                .route(
                    "/federation-whitelist-enabled",
                    MethodRouter::new()
                        .put(set_federation_whitelist_enabled)
                        .get(get_federation_whitelist_enabled)
                        .delete(reset_federation_whitelist_enabled),
                )
                .route(
                    "/federation-friendly-servers",
                    MethodRouter::new()
                        .put(set_federation_friendly_servers)
                        .get(get_federation_friendly_servers)
                        .delete(reset_federation_friendly_servers),
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
                .get(get_server_config)
                // NOTE: `HEAD /v1/server/config` doesn’t require authentication.
                .head(is_server_initialized),
        )
        .with_state(app_state)
}
