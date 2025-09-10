// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{extract::State, response::NoContent, Json};
use axum_extra::either::Either;

use service::{
    auth::IsAdmin,
    models::durations::{DateLike, Duration, PossiblyInfinite},
    prosody_config::ProsodySettings,
    server_config::{
        server_config_controller, PublicServerConfig, ServerConfig, ServerConfigManager, TlsProfile,
    },
    xmpp::JidDomain,
    AppConfig, LinkedHashSet,
};

use crate::{error::Error, extractors::Lua, AppState};

// MARK: Server config

pub mod server_config {
    use super::*;

    pub async fn get(
        State(AppState { ref db, .. }): State<AppState>,
        ref app_config: AppConfig,
        is_admin: Option<IsAdmin>,
    ) -> Result<Either<Json<ServerConfig>, Json<PublicServerConfig>>, Error> {
        match server_config_controller::get_server_config(db, app_config, is_admin).await? {
            service::util::either::Either::E1(config) => Ok(Either::E1(Json(config))),
            service::util::either::Either::E2(config) => Ok(Either::E2(Json(config))),
        }
    }
}

// MARK: Helper macros

/// Generates routes for setting, querying and resetting a specific server config.
macro_rules! server_config_routes {
    ($var:ident: $var_type:ty) => {
        pub mod $var {
            use super::*;

            pub async fn set(
                ref manager: ServerConfigManager,
                Json($var): Json<$var_type>,
            ) -> Result<Json<$var_type>, Error> {
                match server_config_controller::$var::set(manager, $var).await? {
                    $var => Ok(Json($var)),
                }
            }

            pub async fn get(
                State(AppState { ref db, .. }): State<AppState>,
                ref app_config: AppConfig,
            ) -> Result<Json<$var_type>, Error> {
                match server_config_controller::$var::get(db, app_config).await? {
                    $var => Ok(Json($var)),
                }
            }

            pub async fn reset(
                State(AppState { ref db, .. }): State<AppState>,
                ref app_config: AppConfig,
            ) -> Result<Json<$var_type>, Error> {
                server_config_controller::$var::reset(db).await?;
                match server_config_controller::$var::get(db, app_config).await? {
                    $var => Ok(Json($var)),
                }
            }
        }
    };
}

/// Generates a route for resetting a group of server configs.
macro_rules! gen_server_config_group_reset_route {
    ($group:ident) => {
        pub mod $group {
            use axum::http::{header::CONTENT_LOCATION, HeaderName, HeaderValue};

            use crate::features::server_config::SERVER_CONFIG_ROUTE;

            use super::*;

            pub async fn reset(
                ref manager: ServerConfigManager,
            ) -> Result<([(HeaderName, HeaderValue); 1], Json<ServerConfig>), crate::error::Error>
            {
                let new_config = server_config_controller::$group::reset(manager).await?;
                Ok((
                    [(
                        CONTENT_LOCATION,
                        HeaderValue::from_static(SERVER_CONFIG_ROUTE),
                    )],
                    Json(new_config),
                ))
            }
        }
    };
}

// MARK: File upload

gen_server_config_group_reset_route!(files_config);
server_config_routes!(
    file_upload_allowed: bool
);
pub mod file_storage_encryption_scheme {
    use axum::Json;

    pub async fn set(
    ) -> Result<axum::Json<service::server_config::ServerConfig>, crate::error::Error> {
        use service::server_config::server_config_controller;
        match server_config_controller::file_storage_encryption_scheme::set().await? {
            server_config => Ok(Json(server_config)),
        }
    }
}
server_config_routes!(
    file_storage_retention: PossiblyInfinite<Duration<DateLike>>
);

// MARK: Message archive

gen_server_config_group_reset_route!(messaging_config);
server_config_routes!(
    message_archive_enabled: bool
);
server_config_routes!(
    message_archive_retention: PossiblyInfinite<Duration<DateLike>>
);

// MARK: Push notifications

gen_server_config_group_reset_route!(push_notifications_config);
server_config_routes!(
    push_notification_with_body: bool
);
server_config_routes!(
    push_notification_with_sender: bool
);

// MARK: Network encryption

gen_server_config_group_reset_route!(network_encryption_config);
server_config_routes!(
    tls_profile: TlsProfile
);

// MARK: Server federation

gen_server_config_group_reset_route!(server_federation_config);
server_config_routes!(
    federation_enabled: bool
);
server_config_routes!(
    federation_whitelist_enabled: bool
);
server_config_routes!(
    federation_friendly_servers: LinkedHashSet<JidDomain>
);

// MARK: Prosody config (Lua)

pub mod prosody_config_lua {
    use super::*;

    pub async fn get(ref app_config: AppConfig) -> Result<Lua, Error> {
        match server_config_controller::get_prosody_config_lua(app_config).await? {
            prosody_config => Ok(Lua::from(prosody_config)),
        }
    }
}

// MARK: Prosody overrides (JSON)

pub mod prosody_overrides {
    use super::*;

    pub async fn set(
        ref manager: ServerConfigManager,
        Json(overrides): Json<ProsodySettings>,
    ) -> Result<Json<Option<ProsodySettings>>, Error> {
        match server_config_controller::prosody_overrides::set(manager, overrides).await? {
            overrides => Ok(Json(overrides)),
        }
    }

    pub async fn get(
        State(AppState { ref db, .. }): State<AppState>,
        ref app_config: AppConfig,
    ) -> Result<Either<Json<ProsodySettings>, NoContent>, Error> {
        match server_config_controller::prosody_overrides::get(db, app_config).await? {
            Some(overrides) => Ok(Either::E1(Json(overrides))),
            None => Ok(Either::E2(NoContent)),
        }
    }

    pub async fn delete(
        State(AppState { ref db, .. }): State<AppState>,
    ) -> Result<NoContent, Error> {
        match server_config_controller::prosody_overrides::delete(db).await? {
            _ => Ok(NoContent),
        }
    }
}

// MARK: Prosody overrides (raw)

pub mod prosody_overrides_raw {
    use super::*;

    pub async fn set(
        ref manager: ServerConfigManager,
        lua: Lua,
    ) -> Result<Either<Lua, NoContent>, Error> {
        match server_config_controller::prosody_overrides_raw::set(manager, lua.into()).await? {
            Some(overrides) => Ok(Either::E1(Lua::from(overrides))),
            None => Ok(Either::E2(NoContent)),
        }
    }

    pub async fn get(
        State(AppState { ref db, .. }): State<AppState>,
        ref app_config: AppConfig,
    ) -> Result<Either<Lua, NoContent>, Error> {
        match server_config_controller::prosody_overrides_raw::get(db, app_config).await? {
            Some(overrides) => Ok(Either::E1(Lua::from(overrides))),
            None => Ok(Either::E2(NoContent)),
        }
    }

    pub async fn delete(
        State(AppState { ref db, .. }): State<AppState>,
    ) -> Result<NoContent, Error> {
        match server_config_controller::prosody_overrides_raw::delete(db).await? {
            _ => Ok(NoContent),
        }
    }
}
