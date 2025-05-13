// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::State,
    http::{header::IF_MATCH, StatusCode},
    Json,
};
use axum_extra::{either::Either, headers::IfMatch, TypedHeader};

use serde::{Deserialize, Serialize};
use service::{
    auth::IsAdmin,
    models::durations::{DateLike, Duration, PossiblyInfinite},
    server_config::{ServerConfig, ServerConfigRepository, TlsProfile},
    xmpp::JidDomain,
    LinkedHashSet,
};

use crate::{
    error::{Error, PreconditionRequired},
    AppState,
};

// Helper macros

/// Generates routes for setting, querying and resetting a specific server config.
macro_rules! server_config_routes {
    (
        key: $var_name:ident, type: $res_type:ty
        $(,   set:   $set_route_fn:ident using   $set_manager_fn:ident)?
        $(,   get:   $get_route_fn:ident                              )?
        $(, reset: $reset_route_fn:ident using $reset_manager_fn:ident)?
        $(,)?
    ) => {
        $(server_config_route!(  set: $res_type, $var_name,   $set_route_fn,   $set_manager_fn);)?
        $(server_config_route!(  get: $res_type, $var_name,   $get_route_fn                   );)?
        $(server_config_route!(reset: $res_type, $var_name, $reset_route_fn, $reset_manager_fn);)?
    };
}

/// Generates a route for setting, querying or resetting a specific server config.
macro_rules! server_config_route {
    (set: $var_type:ty, $var:ident, $route_fn:ident, $manager_fn:ident) => {
        pub async fn $route_fn(
            server_manager: service::xmpp::ServerManager,
            axum::Json($var): axum::Json<$var_type>,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            server_manager.$manager_fn($var.clone()).await?;
            Ok(axum::Json($var))
        }
    };
    (get: $var_type:ty, $var:ident, $route_fn:ident) => {
        pub async fn $route_fn(
            server_config: service::server_config::ServerConfig,
        ) -> axum::Json<$var_type> {
            axum::Json(server_config.$var)
        }
    };
    (reset: $var_type:ty, $var:ident, $route_fn:ident, $manager_fn:ident) => {
        pub async fn $route_fn(
            server_manager: service::xmpp::ServerManager,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            let $var = server_manager.$manager_fn().await?.$var;
            Ok(axum::Json($var))
        }
    };
}

/// Generates a route for resetting a group of server configs.
macro_rules! server_config_reset_route {
    ($fn:ident, $route_fn:ident $(,)?) => {
        pub async fn $route_fn(
            server_manager: service::xmpp::ServerManager,
        ) -> Result<
            (
                [(axum::http::HeaderName, axum::http::HeaderValue); 1],
                axum::Json<service::server_config::ServerConfig>,
            ),
            crate::error::Error,
        > {
            let new_config = server_manager.$fn().await?;
            Ok((
                [(
                    axum::http::header::CONTENT_LOCATION,
                    axum::http::HeaderValue::from_static(
                        crate::features::init::SERVER_CONFIG_ROUTE,
                    ),
                )],
                axum::Json(new_config),
            ))
        }
    };
}

// Server config

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicServerConfig {
    pub domain: JidDomain,
}

impl From<ServerConfig> for PublicServerConfig {
    fn from(server_config: ServerConfig) -> Self {
        Self {
            domain: server_config.domain,
        }
    }
}

pub async fn get_server_config_route(
    server_config: ServerConfig,
    is_admin: Option<IsAdmin>,
) -> Either<Json<ServerConfig>, Json<PublicServerConfig>> {
    if is_admin.is_some() {
        Either::E1(Json(server_config))
    } else {
        Either::E2(Json(PublicServerConfig::from(server_config)))
    }
}

pub async fn is_server_initialized_route(
    State(AppState { db, .. }): State<AppState>,
    TypedHeader(if_match): TypedHeader<IfMatch>,
) -> Result<StatusCode, Error> {
    if if_match != IfMatch::any() {
        Err(Error::from(PreconditionRequired {
            comment: format!("Missing header: '{IF_MATCH}'."),
        }))
    } else if ServerConfigRepository::is_initialized(&db).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::PRECONDITION_FAILED)
    }
}

// File upload

server_config_reset_route!(reset_files_config, reset_files_config_route);
server_config_routes!(
            key: file_upload_allowed, type: bool,
      set:   set_file_upload_allowed_route using   set_file_upload_allowed,
      get:   get_file_upload_allowed_route,
    reset: reset_file_upload_allowed_route using reset_file_upload_allowed,
);
pub async fn set_file_storage_encryption_scheme_route(
) -> Result<axum::Json<service::server_config::ServerConfig>, crate::error::Error> {
    Err(crate::error::NotImplemented("File storage encryption scheme").into())
}
server_config_routes!(
            key: file_storage_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_file_storage_retention_route using   set_file_storage_retention,
      get:   get_file_storage_retention_route,
    reset: reset_file_storage_retention_route using reset_file_storage_retention,
);

// Message archive

server_config_reset_route!(reset_messaging_config, reset_messaging_config_route);
server_config_routes!(
            key: message_archive_enabled, type: bool,
      set:   set_message_archive_enabled_route using   set_message_archive_enabled,
      get:   get_message_archive_enabled_route,
    reset: reset_message_archive_enabled_route using reset_message_archive_enabled,
);
server_config_routes!(
            key: message_archive_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_message_archive_retention_route using   set_message_archive_retention,
      get:   get_message_archive_retention_route,
    reset: reset_message_archive_retention_route using reset_message_archive_retention,
);

// Push notifications

server_config_reset_route!(
    reset_push_notifications_config,
    reset_push_notifications_config_route,
);
server_config_routes!(
            key: push_notification_with_body, type: bool,
      set:   set_push_notification_with_body_route using   set_push_notification_with_body,
      get:   get_push_notification_with_body_route,
    reset: reset_push_notification_with_body_route using reset_push_notification_with_body,
);
server_config_routes!(
            key: push_notification_with_sender, type: bool,
      set:   set_push_notification_with_sender_route using   set_push_notification_with_sender,
      get:   get_push_notification_with_sender_route,
    reset: reset_push_notification_with_sender_route using reset_push_notification_with_sender,
);

// Network encryption

server_config_reset_route!(
    reset_network_encryption_config,
    reset_network_encryption_config_route,
);
server_config_routes!(
            key: tls_profile, type: TlsProfile,
      set:   set_tls_profile_route using   set_tls_profile,
      get:   get_tls_profile_route,
    reset: reset_tls_profile_route using reset_tls_profile,
);

// Server federation

server_config_reset_route!(
    reset_server_federation_config,
    reset_server_federation_config_route
);
server_config_routes!(
            key: federation_enabled, type: bool,
      set:   set_federation_enabled_route using   set_federation_enabled,
      get:   get_federation_enabled_route,
    reset: reset_federation_enabled_route using reset_federation_enabled,
);
server_config_routes!(
            key: federation_whitelist_enabled, type: bool,
      set:   set_federation_whitelist_enabled_route using   set_federation_whitelist_enabled,
      get:   get_federation_whitelist_enabled_route,
    reset: reset_federation_whitelist_enabled_route using reset_federation_whitelist_enabled,
);
server_config_routes!(
            key: federation_friendly_servers, type: LinkedHashSet<String>,
      set:   set_federation_friendly_servers_route using   set_federation_friendly_servers,
      get:   get_federation_friendly_servers_route,
    reset: reset_federation_friendly_servers_route using reset_federation_friendly_servers,
);
