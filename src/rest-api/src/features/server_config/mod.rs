// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod guards;
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
                .route("/files", delete(files_config::reset))
                .route(
                    "/file-upload-allowed",
                    MethodRouter::new()
                        .put(file_upload_allowed::set)
                        .get(file_upload_allowed::get)
                        .delete(file_upload_allowed::reset),
                )
                .route(
                    "/file-storage-encryption-scheme",
                    put(file_storage_encryption_scheme::set),
                )
                .route(
                    "/file-storage-retention",
                    MethodRouter::new()
                        .put(file_storage_retention::set)
                        .get(file_storage_retention::get)
                        .delete(file_storage_retention::reset),
                )
                // Message archive
                .route("/messaging", delete(messaging_config::reset))
                .route(
                    "/message-archive-enabled",
                    MethodRouter::new()
                        .put(message_archive_enabled::set)
                        .get(message_archive_enabled::get)
                        .delete(message_archive_enabled::reset),
                )
                .route(
                    "/message-archive-retention",
                    MethodRouter::new()
                        .put(message_archive_retention::set)
                        .get(message_archive_retention::get)
                        .delete(message_archive_retention::reset),
                )
                // Push notifications
                .route(
                    "/push-notifications",
                    delete(push_notifications_config::reset),
                )
                .route(
                    "/push-notification-with-body",
                    MethodRouter::new()
                        .put(push_notification_with_body::set)
                        .get(push_notification_with_body::get)
                        .delete(push_notification_with_body::reset),
                )
                .route(
                    "/push-notification-with-sender",
                    MethodRouter::new()
                        .put(push_notification_with_sender::set)
                        .get(push_notification_with_sender::get)
                        .delete(push_notification_with_sender::reset),
                )
                // Network encryption
                .route(
                    "/network-encryption",
                    delete(network_encryption_config::reset),
                )
                .route(
                    "/tls-profile",
                    MethodRouter::new()
                        .put(tls_profile::set)
                        .get(tls_profile::get)
                        .delete(tls_profile::reset),
                )
                // Server federation
                .route(
                    "/server-federation",
                    delete(server_federation_config::reset),
                )
                .route(
                    "/federation-enabled",
                    MethodRouter::new()
                        .put(federation_enabled::set)
                        .get(federation_enabled::get)
                        .delete(federation_enabled::reset),
                )
                .route(
                    "/federation-whitelist-enabled",
                    MethodRouter::new()
                        .put(federation_whitelist_enabled::set)
                        .get(federation_whitelist_enabled::get)
                        .delete(federation_whitelist_enabled::reset),
                )
                .route(
                    "/federation-friendly-servers",
                    MethodRouter::new()
                        .put(federation_friendly_servers::set)
                        .get(federation_friendly_servers::get)
                        .delete(federation_friendly_servers::reset),
                )
                // Prosody
                .route("/prosody", get(prosody_config_lua::get))
                .route(
                    "/prosody-overrides",
                    MethodRouter::new()
                        .put(prosody_overrides::set)
                        .get(prosody_overrides::get)
                        .delete(prosody_overrides::delete),
                )
                .route(
                    "/prosody-overrides-raw",
                    MethodRouter::new()
                        .put(prosody_overrides_raw::set)
                        .get(prosody_overrides_raw::get)
                        .delete(prosody_overrides_raw::delete),
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
                .get(server_config::get),
        )
        .with_state(app_state)
}
