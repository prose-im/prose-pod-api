// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::entity::prelude::*;

use crate::{
    app_config::{AppConfig, ConfigServerDefaults},
    models::{durations::*, sea_orm::LinkedStringSet, xmpp::*},
    server_config::{ServerConfig, TlsProfile},
};

/// XMPP server configuration, as stored in the database.
///
/// All fields are optional because the Prose Pod API only stores manual overrides.
/// This way, if security defaults are raised, every Prose Pod will automatically benefit from it upon update.
/// Those default values (from [config::defaults][crate::config::defaults]) can also be overridden
/// by a Prose Pod administrator via the Prose Pod API configuration file (`Prose.toml`).
///
/// When returning the server configuration, the Prose Pod API replaces non-overridden (empty) values
/// by their default. See [ServerConfig] and [Model::with_default_values].
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "server_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    pub domain: JidDomain,
    pub message_archive_enabled: Option<bool>,
    pub message_archive_retention: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub file_upload_allowed: Option<bool>,
    pub file_storage_encryption_scheme: Option<String>,
    pub file_storage_retention: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub mfa_required: Option<bool>,
    pub tls_profile: Option<TlsProfile>,
    pub federation_enabled: Option<bool>,
    pub federation_whitelist_enabled: Option<bool>,
    pub federation_friendly_servers: Option<LinkedStringSet>,
    pub settings_backup_interval: Option<String>,
    pub user_data_backup_interval: Option<String>,
    pub push_notification_with_body: Option<bool>,
    pub push_notification_with_sender: Option<bool>,
    pub prosody_overrides: Option<Json>,
    pub prosody_overrides_raw: Option<String>,
}

impl Model {
    pub fn with_default_values(&self, defaults: &ConfigServerDefaults) -> ServerConfig {
        // NOTE: Destructure so the compiler checks we will never miss using a field.
        let ConfigServerDefaults {
            message_archive_enabled,
            message_archive_retention,
            file_upload_allowed,
            file_storage_encryption_scheme,
            file_storage_retention,
            mfa_required,
            tls_profile,
            federation_enabled,
            federation_whitelist_enabled,
            federation_friendly_servers,
            settings_backup_interval,
            user_data_backup_interval,
            push_notification_with_body,
            push_notification_with_sender,
        } = defaults;

        macro_rules! get_or_default {
            ($var:ident) => {
                self.$var.unwrap_or($var.to_owned())
            };
            (deref $var:ident) => {
                self.$var.as_deref().unwrap_or($var).to_owned()
            };
        }

        ServerConfig {
            domain: self.domain.to_owned(),
            message_archive_enabled: get_or_default!(message_archive_enabled),
            message_archive_retention: get_or_default!(message_archive_retention),
            file_upload_allowed: get_or_default!(file_upload_allowed),
            file_storage_encryption_scheme: get_or_default!(deref file_storage_encryption_scheme),
            file_storage_retention: get_or_default!(file_storage_retention),
            mfa_required: get_or_default!(mfa_required),
            tls_profile: get_or_default!(tls_profile),
            federation_enabled: get_or_default!(federation_enabled),
            federation_whitelist_enabled: get_or_default!(federation_whitelist_enabled),
            federation_friendly_servers: get_or_default!(deref federation_friendly_servers),
            settings_backup_interval: get_or_default!(deref settings_backup_interval),
            user_data_backup_interval: get_or_default!(deref user_data_backup_interval),
            push_notification_with_body: get_or_default!(push_notification_with_body),
            push_notification_with_sender: get_or_default!(push_notification_with_sender),
            c2s_unencrypted: false,
            prosody_overrides: self.prosody_overrides.to_owned(),
            prosody_overrides_raw: self.prosody_overrides_raw.to_owned(),
        }
    }
    /// Same as [Model::with_default_values], used in places where we have easier access to a full [AppConfig].
    pub fn with_default_values_from(&self, app_config: &AppConfig) -> ServerConfig {
        let mut config = self.with_default_values(&app_config.server.defaults);
        config.c2s_unencrypted = app_config.debug.c2s_unencrypted;
        config
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
