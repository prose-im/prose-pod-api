// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Read as _};

use anyhow::Context;
use sea_orm::DatabaseConnection;

use crate::{
    auth::{AuthService, IsAdmin},
    models::{
        durations::{DateLike, Duration, PossiblyInfinite},
        Lua,
    },
    prosody::ProsodyOverrides,
    secrets::SecretsStore,
    server_config::{ServerConfig, ServerConfigRepository, TlsProfile},
    util::Either,
    xmpp::{JidDomain, ServerCtl, ServerManager},
    AppConfig, LinkedHashSet,
};

use super::{errors::*, server_config, ServerConfigCreateForm};

// Helper macros

/// Generates routes for setting, querying and resetting a specific server config.
macro_rules! gen_field_routes {
    (
        key: $var_name:ident, type: $var_type:ty
        $(,   set:   $set_fn:ident)?
        $(,   get:   $get_fn:ident)?
        $(, reset: $reset_fn:ident)?
        $(,)?
    ) => {
        $(pub async fn $set_fn(
            server_manager: &crate::xmpp::ServerManager,
            $var_name: $var_type,
        ) -> anyhow::Result<$var_type> {
            match server_manager.$set_fn($var_name.clone()).await? {
                _ => Ok($var_name),
            }
        })?

        $(pub async fn $get_fn(server_config: crate::server_config::ServerConfig) -> $var_type {
            server_config.$var_name
        })?

        $(pub async fn $reset_fn(server_manager: &crate::xmpp::ServerManager) -> anyhow::Result<$var_type> {
            match server_manager.$reset_fn().await?.$var_name {
                $var_name => Ok($var_name),
            }
        })?
    };
}

/// Generates a route for resetting a group of server configs.
macro_rules! gen_server_config_group_reset_route {
    ($fn:ident) => {
        pub async fn $fn(
            server_manager: &crate::xmpp::ServerManager,
        ) -> anyhow::Result<crate::server_config::ServerConfig> {
            server_manager.$fn().await
        }
    };
}

// MARK: INIT SERVER CONFIG

#[tracing::instrument(level = "trace", skip_all, err(level = "trace"))]
pub async fn init_server_config(
    db: &DatabaseConnection,
    server_ctl: &ServerCtl,
    app_config: &AppConfig,
    auth_service: &AuthService,
    secrets_store: &SecretsStore,
    form: impl Into<ServerConfigCreateForm>,
) -> Result<ServerConfig, Either<ServerConfigAlreadyInitialized, anyhow::Error>> {
    // Initialize XMPP server configuration.
    let server_config = ServerManager::init_server_config(db, server_ctl, app_config, form).await?;

    // Register OAuth 2.0 client.
    (auth_service.register_oauth2_client().await).context("Could not register OAuth 2.0 client")?;

    // Create service XMPP accounts.
    ServerManager::create_service_accounts(
        &server_config.domain,
        server_ctl,
        app_config,
        auth_service,
        secrets_store,
    )
    .await
    .context("Could not create service XMPP account")?;

    // Add the Workspace XMPP account to everyone’s rosters so they receive
    // Workspace icon updates.
    let workspace_jid = app_config.workspace_jid(&server_config.domain);
    (server_ctl.add_team_member(&workspace_jid).await)
        .context("Could not add the Workspace to the team")?;

    Ok(server_config)
}

pub async fn is_server_initialized(db: &DatabaseConnection) -> anyhow::Result<bool> {
    (ServerConfigRepository::is_initialized(db).await).context("Database error")
}

// Server config

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicServerConfig {
    pub domain: JidDomain,
}

pub async fn get_server_domain(db: &DatabaseConnection) -> anyhow::Result<Option<JidDomain>> {
    match ServerConfigRepository::get(db).await? {
        Some(model) => Ok(Some(model.domain)),
        None => Ok(None),
    }
}
pub async fn get_server_config(
    db: &DatabaseConnection,
    app_config: &AppConfig,
    _is_admin: &IsAdmin,
) -> anyhow::Result<Option<ServerConfig>> {
    match ServerConfigRepository::get(db).await? {
        Some(model) => {
            let server_config = model.with_default_values_from(app_config);
            Ok(Some(server_config))
        }
        None => Ok(None),
    }
}
pub async fn get_server_config_public(
    db: &DatabaseConnection,
    app_config: &AppConfig,
    is_admin: Option<IsAdmin>,
) -> anyhow::Result<Option<Either<ServerConfig, PublicServerConfig>>> {
    match ServerConfigRepository::get(db).await? {
        Some(model) => {
            if is_admin.is_some() {
                let server_config = model.with_default_values_from(app_config);
                Ok(Some(Either::E1(server_config)))
            } else {
                Ok(Some(Either::E2(PublicServerConfig::from(model))))
            }
        }
        None => Ok(None),
    }
}

// File upload

gen_server_config_group_reset_route!(reset_files_config);
gen_field_routes!(
            key: file_upload_allowed, type: bool,
      set:   set_file_upload_allowed,
      get:   get_file_upload_allowed,
    reset: reset_file_upload_allowed,
);
pub async fn set_file_storage_encryption_scheme(
) -> Result<crate::server_config::ServerConfig, crate::errors::NotImplemented> {
    Err(crate::errors::NotImplemented(
        "File storage encryption scheme",
    ))
}
gen_field_routes!(
            key: file_storage_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_file_storage_retention,
      get:   get_file_storage_retention,
    reset: reset_file_storage_retention,
);

// Message archive

gen_server_config_group_reset_route!(reset_messaging_config);
gen_field_routes!(
            key: message_archive_enabled, type: bool,
      set:   set_message_archive_enabled,
      get:   get_message_archive_enabled,
    reset: reset_message_archive_enabled,
);
gen_field_routes!(
            key: message_archive_retention, type: PossiblyInfinite<Duration<DateLike>>,
      set:   set_message_archive_retention,
      get:   get_message_archive_retention,
    reset: reset_message_archive_retention,
);

// Push notifications

gen_server_config_group_reset_route!(reset_push_notifications_config);
gen_field_routes!(
            key: push_notification_with_body, type: bool,
      set:   set_push_notification_with_body,
      get:   get_push_notification_with_body,
    reset: reset_push_notification_with_body,
);
gen_field_routes!(
            key: push_notification_with_sender, type: bool,
      set:   set_push_notification_with_sender,
      get:   get_push_notification_with_sender,
    reset: reset_push_notification_with_sender,
);

// Network encryption

gen_server_config_group_reset_route!(reset_network_encryption_config);
gen_field_routes!(
            key: tls_profile, type: TlsProfile,
      set:   set_tls_profile,
      get:   get_tls_profile,
    reset: reset_tls_profile,
);

// Server federation

gen_server_config_group_reset_route!(reset_server_federation_config);
gen_field_routes!(
            key: federation_enabled, type: bool,
      set:   set_federation_enabled,
      get:   get_federation_enabled,
    reset: reset_federation_enabled,
);
gen_field_routes!(
            key: federation_whitelist_enabled, type: bool,
      set:   set_federation_whitelist_enabled,
      get:   get_federation_whitelist_enabled,
    reset: reset_federation_whitelist_enabled,
);
gen_field_routes!(
            key: federation_friendly_servers, type: LinkedHashSet<String>,
      set:   set_federation_friendly_servers,
      get:   get_federation_friendly_servers,
    reset: reset_federation_friendly_servers,
);

// GET PROSODY CONFIG (LUA)

pub async fn get_prosody_config_lua(app_config: &AppConfig) -> anyhow::Result<Lua> {
    let config_file_path = &app_config.prosody_ext.config_file_path;
    let mut file = File::open(config_file_path).map_err(|err| {
        anyhow::Error::new(err).context(format!(
            "Cannot open Prosody config file at `{path}`",
            path = config_file_path.display(),
        ))
    })?;

    let mut prosody_config = String::new();
    file.read_to_string(&mut prosody_config).map_err(|err| {
        anyhow::Error::new(err).context(format!(
            "Cannot read Prosody config file at `{path}`",
            path = config_file_path.display(),
        ))
    })?;

    Ok(Lua(prosody_config))
}

// PROSODY OVERRIDES (JSON)

pub async fn set_prosody_overrides(
    server_manager: &ServerManager,
    overrides: ProsodyOverrides,
) -> anyhow::Result<Option<ProsodyOverrides>> {
    let res = match server_manager.set_prosody_overrides(overrides).await? {
        // NOTE: It’s safe enough to force unwrap here as the
        //   JSON data should be exactly the user’s request.
        model => (model.prosody_overrides).map(|json| serde_json::from_value(json).unwrap()),
    };

    server_manager.reload_current().await?;

    Ok(res)
}

pub async fn get_prosody_overrides(
    server_manager: &ServerManager,
) -> anyhow::Result<Option<Option<ProsodyOverrides>>> {
    (server_manager.get_prosody_overrides().await)
        .context("Prosody overrides stored in database cannot be read. To fix this, call `PUT /v1/server/config/prosody-overrides` with a new value. You can `GET /v1/server/config/prosody-overrides` first to see what the stored value was.")
}

pub async fn delete_prosody_overrides(
    server_manager: &ServerManager,
) -> anyhow::Result<ServerConfig> {
    server_manager.reset_prosody_overrides().await
}

// PROSODY OVERRIDES (RAW)

pub async fn set_prosody_overrides_raw(
    server_manager: &ServerManager,
    Lua(overrides): Lua,
) -> anyhow::Result<Lua> {
    let res = match server_manager.set_prosody_overrides_raw(overrides).await? {
        // NOTE: It’s safe enough to force unwrap here as the
        //   Lua data should be exactly the user’s request.
        model => model.prosody_overrides_raw.map(Lua).unwrap(),
    };

    server_manager.reload_current().await?;

    Ok(res)
}

pub async fn get_prosody_overrides_raw(
    server_manager: &ServerManager,
) -> anyhow::Result<Option<Lua>> {
    match server_manager.get_prosody_overrides_raw().await? {
        opt => Ok(opt.map(Lua)),
    }
}

pub async fn delete_prosody_overrides_raw(
    server_manager: &ServerManager,
) -> anyhow::Result<ServerConfig> {
    server_manager.reset_prosody_overrides().await
}

// MARK: BOILERPLATE

impl From<server_config::Model> for PublicServerConfig {
    fn from(server_config: server_config::Model) -> Self {
        Self {
            domain: server_config.domain,
        }
    }
}

impl From<ServerConfig> for PublicServerConfig {
    fn from(server_config: ServerConfig) -> Self {
        Self {
            domain: server_config.domain,
        }
    }
}
