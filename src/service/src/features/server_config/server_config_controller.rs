// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fs::File, io::Read as _};

use sea_orm::DatabaseConnection;

use crate::{
    auth::IsAdmin,
    models::{durations::*, sea_orm::LinkedStringSet, Lua},
    prosody_config::ProsodySettings,
    server_config::{ServerConfig, TlsProfile},
    util::either::Either,
    xmpp::JidDomain,
    AppConfig, LinkedHashSet,
};

use super::{server_config_repository, PublicServerConfig};

// MARK: Server config

#[inline]
pub async fn get_server_domain(app_config: &AppConfig) -> JidDomain {
    app_config.server.domain.clone()
}
#[inline]
pub async fn get_server_config(
    db: &DatabaseConnection,
    app_config: &AppConfig,
    _is_admin: &IsAdmin,
) -> anyhow::Result<ServerConfig> {
    let ref dynamic_server_config = server_config_repository::get(db).await?;
    let server_config = ServerConfig::with_default_values(dynamic_server_config, app_config);
    Ok(server_config)
}
pub async fn get_server_config_public(
    db: &DatabaseConnection,
    app_config: &AppConfig,
    is_admin: Option<IsAdmin>,
) -> anyhow::Result<Either<ServerConfig, PublicServerConfig>> {
    if let Some(ref is_admin) = is_admin {
        (get_server_config(db, app_config, is_admin).await).map(Either::E1)
    } else {
        Ok(Either::E2(PublicServerConfig::from(app_config)))
    }
}

// MARK: Helper macros

/// Generates routes for setting, querying and resetting a specific server config.
macro_rules! gen_field_routes {
    ($var_name:ident: $var_type:ty $(as $db_type:ty)? $([$extra_fn:ident])?) => {
        pub mod $var_name {
            #[allow(unused)]
            use super::*;

            pub async fn set(
                manager: &crate::server_config::ServerConfigManager,
                new_state: $var_type,
            ) -> anyhow::Result<$var_type> {
                $(let new_state = <$db_type>::from(new_state);)?
                tracing::trace!("Setting {} to {new_state}…", stringify!($var_name));
                let var = manager
                    .update(|txn| async move {
                        crate::server_config::$var_name::set(&txn, new_state).await?;
                        Ok(txn)
                    })
                    .await?
                    .$var_name;
                Ok(var$(.$extra_fn())?)
            }

            pub async fn get(
                db: &sea_orm::DatabaseConnection,
                app_config: &crate::AppConfig,
            ) -> anyhow::Result<gen_field_routes!(ret $var_type $([$extra_fn])?)> {
                tracing::trace!("Getting {}…", stringify!($var_name));
                let var = crate::server_config::$var_name::get_opt(db).await?;
                let var = crate::ServerConfig::with_default_values(
                    &crate::server_config::DynamicServerConfig {
                        $var_name: var,
                        ..Default::default()
                    },
                    app_config,
                ).$var_name;
                Ok(var)
            }

            pub async fn reset(
                db: &sea_orm::DatabaseConnection,
            ) -> anyhow::Result<()> {
                crate::server_config::$var_name::delete(db).await
            }
        }
    };
    // NOTE: Internal.
    (ret $t:ty [unwrap]) => {
        Option<$t>
    };
    // NOTE: Internal.
    (ret $t:ty) => {
        $t
    };
}

/// Generates a route for resetting a group of server configs.
macro_rules! gen_server_config_group_reset_route {
    ($group:ident: $($field:ident)+) => {
        pub mod $group {
            use crate::server_config::{self, ServerConfig, ServerConfigManager};

            pub async fn reset(manager: &ServerConfigManager) -> anyhow::Result<ServerConfig> {
                tracing::trace!("Resetting config group {}…", stringify!($group));
                manager
                    .update(|txn| async {
                        $(server_config::$field::delete(&txn).await?;)+
                        Ok(txn)
                    })
                    .await
            }
        }
    };
}

// MARK: File upload

gen_server_config_group_reset_route!(
    files_config:
    file_upload_allowed
    file_storage_encryption_scheme
    file_storage_retention
);
gen_field_routes!(
    file_upload_allowed: bool
);
pub mod file_storage_encryption_scheme {
    pub async fn set() -> Result<crate::server_config::ServerConfig, crate::errors::NotImplemented>
    {
        Err(crate::errors::NotImplemented(
            "File storage encryption scheme",
        ))
    }
}
gen_field_routes!(
    file_storage_retention: PossiblyInfinite<Duration<DateLike>>
);

// MARK: Message archive

gen_server_config_group_reset_route!(
    messaging_config:
    message_archive_enabled
    message_archive_retention
);
gen_field_routes!(
    message_archive_enabled: bool
);
gen_field_routes!(
    message_archive_retention: PossiblyInfinite<Duration<DateLike>>
);

// MARK: Push notifications

gen_server_config_group_reset_route!(
    push_notifications_config:
    push_notification_with_body
    push_notification_with_sender
);
gen_field_routes!(
    push_notification_with_body: bool
);
gen_field_routes!(
    push_notification_with_sender: bool
);

// MARK: Network encryption

gen_server_config_group_reset_route!(
    network_encryption_config:
    tls_profile
);
gen_field_routes!(
    tls_profile: TlsProfile
);

// MARK: Server federation

gen_server_config_group_reset_route!(
    server_federation_config:
    federation_enabled
    federation_whitelist_enabled
    federation_friendly_servers
);
gen_field_routes!(
    federation_enabled: bool
);
gen_field_routes!(
    federation_whitelist_enabled: bool
);
gen_field_routes!(
    federation_friendly_servers: LinkedHashSet<String> as LinkedStringSet
);

// MARK: Prosody config (Lua)

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

// MARK: Prosody overrides (JSON)

pub mod prosody_overrides {
    #[allow(unused)]
    use super::*;

    pub async fn set(
        manager: &crate::server_config::ServerConfigManager,
        new_state: ProsodySettings,
    ) -> anyhow::Result<Option<ProsodySettings>> {
        tracing::trace!("Setting prosody_overrides to {new_state:?}…");
        let new_val = manager
            .update(|txn| async move {
                crate::server_config::prosody_overrides::set(&txn, new_state).await?;
                Ok(txn)
            })
            .await?
            .prosody_overrides;
        Ok(new_val)
    }

    pub async fn get(
        db: &sea_orm::DatabaseConnection,
        app_config: &crate::AppConfig,
    ) -> anyhow::Result<Option<ProsodySettings>> {
        tracing::trace!("Getting {}…", stringify!(prosody_overrides));
        let val = crate::server_config::prosody_overrides::get_opt(db).await?;
        if val.is_none() {
            use super::server_config_repository::{kv_store, prosody_overrides};
            if (kv_store::has_key(db, prosody_overrides::KEY).await).unwrap_or_default() {
                tracing::warn!("Prosody overrides stored in database cannot be read. To fix this, call `PUT /v1/server/config/prosody-overrides` with a new value. You can `GET /v1/server/config/prosody-overrides` first to see what the stored value was.")
            }
        }
        let val = ServerConfig::with_default_values(
            &crate::server_config::DynamicServerConfig {
                prosody_overrides: val,
                ..Default::default()
            },
            app_config,
        )
        .prosody_overrides;
        Ok(val)
    }

    pub async fn delete(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
        crate::server_config::prosody_overrides::delete(db).await
    }
}

// MARK: Prosody overrides (raw)

pub mod prosody_overrides_raw {
    #[allow(unused)]
    use super::*;

    pub async fn set(
        manager: &crate::server_config::ServerConfigManager,
        new_state: Lua,
    ) -> anyhow::Result<Option<Lua>> {
        tracing::trace!("Setting prosody_overrides_raw to {new_state:?}…");
        let new_val = manager
            .update(|txn| async move {
                crate::server_config::prosody_overrides_raw::set(&txn, new_state).await?;
                Ok(txn)
            })
            .await?
            .prosody_overrides_raw;
        Ok(new_val)
    }

    pub async fn get(
        db: &sea_orm::DatabaseConnection,
        app_config: &crate::AppConfig,
    ) -> anyhow::Result<Option<Lua>> {
        tracing::trace!("Getting {}…", stringify!(prosody_overrides_raw));
        let var = crate::server_config::prosody_overrides_raw::get_opt(db).await?;
        let var = ServerConfig::with_default_values(
            &crate::server_config::DynamicServerConfig {
                prosody_overrides_raw: var,
                ..Default::default()
            },
            app_config,
        )
        .prosody_overrides_raw;
        Ok(var)
    }

    pub async fn delete(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
        crate::server_config::prosody_overrides_raw::delete(db).await
    }
}

// MARK: - Boilerplate

impl From<&AppConfig> for PublicServerConfig {
    fn from(app_config: &AppConfig) -> Self {
        Self {
            domain: app_config.server.domain.clone(),
        }
    }
}
