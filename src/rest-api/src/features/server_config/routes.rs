// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::State,
    http::{header::IF_MATCH, HeaderValue, StatusCode},
    response::NoContent,
    Json,
};
use axum_extra::{either::Either, headers::IfMatch, TypedHeader};

use service::{
    auth::{AuthService, IsAdmin},
    models::durations::{DateLike, Duration, PossiblyInfinite},
    prosody::ProsodyOverrides,
    secrets::SecretsStore,
    server_config::{
        errors::ServerConfigNotInitialized,
        server_config_controller::{self, PublicServerConfig},
        ServerConfig, TlsProfile,
    },
    xmpp::{ServerCtl, ServerManager},
    AppConfig, LinkedHashSet,
};

use crate::{
    error::{Error, PreconditionRequired},
    guards::Lua,
    responders::Created,
    AppState,
};

use super::{dtos::InitServerConfigRequest, SERVER_CONFIG_ROUTE};

// Helper macros

/// Generates routes for setting, querying and resetting a specific server config.
macro_rules! server_config_routes {
    (
        key: $var_name:ident, type: $res_type:ty
        $(,   set:   $set_route_fn:ident)?
        $(,   get:   $get_route_fn:ident)?
        $(, reset: $reset_route_fn:ident)?
        $(,)?
    ) => {
        $(server_config_route!(  set: $res_type, $var_name,   $set_route_fn);)?
        $(server_config_route!(  get: $res_type, $var_name,   $get_route_fn);)?
        $(server_config_route!(reset: $res_type, $var_name, $reset_route_fn);)?
    };
}

/// Generates a route for setting, querying or resetting a specific server config.
macro_rules! server_config_route {
    (set: $var_type:ty, $var:ident, $fn:ident) => {
        pub async fn $fn(
            ref server_manager: service::xmpp::ServerManager,
            axum::Json($var): axum::Json<$var_type>,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            use service::server_config::server_config_controller;
            match server_config_controller::$fn(server_manager, $var).await? {
                $var => Ok(axum::Json($var)),
            }
        }
    };
    (get: $var_type:ty, $var:ident, $fn:ident) => {
        pub async fn $fn(
            axum::extract::State(crate::AppState { ref db, .. }): axum::extract::State<
                crate::AppState,
            >,
            ref app_config: service::AppConfig,
            ref is_admin: service::auth::IsAdmin,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            use crate::error::Error;
            use service::server_config::{
                errors::ServerConfigNotInitialized, server_config_controller,
            };

            match server_config_controller::get_server_config(db, app_config, is_admin).await {
                Ok(Some(server_config)) => Ok(axum::Json(server_config.$var)),
                Ok(None) => Err(Error::from(ServerConfigNotInitialized)),
                Err(err) => Err(Error::from(err)),
            }
        }
    };
    (reset: $var_type:ty, $var:ident, $fn:ident) => {
        pub async fn $fn(
            ref server_manager: service::xmpp::ServerManager,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            use service::server_config::server_config_controller;
            match server_config_controller::$fn(server_manager).await? {
                $var => Ok(axum::Json($var)),
            }
        }
    };
}

/// Generates a route for resetting a group of server configs.
macro_rules! gen_server_config_group_reset_route {
    ($fn:ident) => {
        pub async fn $fn(
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
                        crate::features::server_config::SERVER_CONFIG_ROUTE,
                    ),
                )],
                axum::Json(new_config),
            ))
        }
    };
}

// MARK: INIT SERVER CONFIG

pub async fn init_server_config_route(
    State(AppState { ref db, .. }): State<AppState>,
    ref server_ctl: ServerCtl,
    ref app_config: AppConfig,
    ref auth_service: AuthService,
    ref secrets_store: SecretsStore,
    Json(req): Json<InitServerConfigRequest>,
) -> Result<Created<ServerConfig>, Error> {
    let server_config = server_config_controller::init_server_config(
        db,
        server_ctl,
        app_config,
        auth_service,
        secrets_store,
        req,
    )
    .await?;

    Ok(Created {
        location: HeaderValue::from_static(SERVER_CONFIG_ROUTE),
        body: server_config,
    })
}

// Server config

pub async fn get_server_config(
    State(AppState { ref db, .. }): State<AppState>,
    ref app_config: AppConfig,
    is_admin: Option<IsAdmin>,
) -> Result<Either<Json<ServerConfig>, Json<PublicServerConfig>>, Error> {
    match server_config_controller::get_server_config_public(db, app_config, is_admin).await? {
        Some(service::util::either::Either::E1(config)) => Ok(Either::E1(Json(config))),
        Some(service::util::either::Either::E2(config)) => Ok(Either::E2(Json(config))),
        None => Err(Error::from(ServerConfigNotInitialized)),
    }
}

pub async fn is_server_initialized(
    State(AppState { ref db, .. }): State<AppState>,
    TypedHeader(if_match): TypedHeader<IfMatch>,
) -> Result<StatusCode, Error> {
    if if_match != IfMatch::any() {
        Err(Error::from(PreconditionRequired {
            comment: format!("Missing header: '{IF_MATCH}'."),
        }))
    } else if server_config_controller::is_server_initialized(db).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::PRECONDITION_FAILED)
    }
}

// File upload

gen_server_config_group_reset_route!(reset_files_config);
server_config_routes!(
            key: file_upload_allowed, type: bool,
      set:   set_file_upload_allowed,
      get:   get_file_upload_allowed,
    reset: reset_file_upload_allowed,
);
pub async fn set_file_storage_encryption_scheme(
) -> Result<axum::Json<service::server_config::ServerConfig>, crate::error::Error> {
    use service::server_config::server_config_controller;
    match server_config_controller::set_file_storage_encryption_scheme().await? {
        server_config => Ok(Json(server_config)),
    }
}
server_config_routes!(
            key: file_storage_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_file_storage_retention,
      get:   get_file_storage_retention,
    reset: reset_file_storage_retention,
);

// Message archive

gen_server_config_group_reset_route!(reset_messaging_config);
server_config_routes!(
            key: message_archive_enabled, type: bool,
      set:   set_message_archive_enabled,
      get:   get_message_archive_enabled,
    reset: reset_message_archive_enabled,
);
server_config_routes!(
            key: message_archive_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_message_archive_retention,
      get:   get_message_archive_retention,
    reset: reset_message_archive_retention,
);

// Push notifications

gen_server_config_group_reset_route!(reset_push_notifications_config);
server_config_routes!(
            key: push_notification_with_body, type: bool,
      set:   set_push_notification_with_body,
      get:   get_push_notification_with_body,
    reset: reset_push_notification_with_body,
);
server_config_routes!(
            key: push_notification_with_sender, type: bool,
      set:   set_push_notification_with_sender,
      get:   get_push_notification_with_sender,
    reset: reset_push_notification_with_sender,
);

// Network encryption

gen_server_config_group_reset_route!(reset_network_encryption_config);
server_config_routes!(
            key: tls_profile, type: TlsProfile,
      set:   set_tls_profile,
      get:   get_tls_profile,
    reset: reset_tls_profile,
);

// Server federation

gen_server_config_group_reset_route!(reset_server_federation_config);
server_config_routes!(
            key: federation_enabled, type: bool,
      set:   set_federation_enabled,
      get:   get_federation_enabled,
    reset: reset_federation_enabled,
);
server_config_routes!(
            key: federation_whitelist_enabled, type: bool,
      set:   set_federation_whitelist_enabled,
      get:   get_federation_whitelist_enabled,
    reset: reset_federation_whitelist_enabled,
);
server_config_routes!(
            key: federation_friendly_servers, type: LinkedHashSet<String>,
      set:   set_federation_friendly_servers,
      get:   get_federation_friendly_servers,
    reset: reset_federation_friendly_servers,
);

// GET PROSODY CONFIG (LUA)

pub async fn get_prosody_config_lua_route(ref app_config: AppConfig) -> Result<Lua, Error> {
    match server_config_controller::get_prosody_config_lua(app_config).await? {
        prosody_config => Ok(Lua::from(prosody_config)),
    }
}

// PROSODY OVERRIDES (JSON)

pub async fn set_prosody_overrides_route(
    ref server_manager: ServerManager,
    Json(overrides): Json<ProsodyOverrides>,
) -> Result<Json<Option<ProsodyOverrides>>, Error> {
    match server_config_controller::set_prosody_overrides(server_manager, overrides).await? {
        overrides => Ok(Json(overrides)),
    }
}

pub async fn get_prosody_overrides_route(
    ref server_manager: ServerManager,
) -> Result<Either<Json<ProsodyOverrides>, NoContent>, Error> {
    match server_config_controller::get_prosody_overrides(server_manager).await? {
        Some(Some(overrides)) => Ok(Either::E1(Json(overrides))),
        Some(None) => Ok(Either::E2(NoContent)),
        None => Err(Error::from(ServerConfigNotInitialized)),
    }
}

pub async fn delete_prosody_overrides_route(
    ref server_manager: ServerManager,
) -> Result<NoContent, Error> {
    match server_config_controller::delete_prosody_overrides(server_manager).await? {
        _ => Ok(NoContent),
    }
}

// PROSODY OVERRIDES (RAW)

pub async fn set_prosody_overrides_raw_route(
    ref server_manager: ServerManager,
    lua: Lua,
) -> Result<Lua, Error> {
    match server_config_controller::set_prosody_overrides_raw(server_manager, lua.into()).await? {
        lua => Ok(Lua::from(lua)),
    }
}

pub async fn get_prosody_overrides_raw_route(
    ref server_manager: ServerManager,
) -> Result<Either<Lua, NoContent>, Error> {
    match server_config_controller::get_prosody_overrides_raw(server_manager).await? {
        Some(overrides) => Ok(Either::E1(Lua::from(overrides))),
        None => Ok(Either::E2(NoContent)),
    }
}

pub async fn delete_prosody_overrides_raw_route(
    ref server_manager: ServerManager,
) -> Result<NoContent, Error> {
    match server_config_controller::delete_prosody_overrides_raw(server_manager).await? {
        _ => Ok(NoContent),
    }
}
