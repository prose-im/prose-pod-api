// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_set::LinkedHashSet;
use prosody_config::ProsodySettings;
use serde_with::{serde_as, DefaultOnError};

use crate::{
    app_config::ServerConfigDefaults,
    models::{durations::*, sea_orm::LinkedStringSet, JidDomain, Lua},
    AppConfig,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicServerConfig {
    pub domain: JidDomain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[serde_with::skip_serializing_none]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub domain: JidDomain,
    pub message_archive_enabled: bool,
    pub message_archive_retention: PossiblyInfinite<Duration<DateLike>>,
    pub file_upload_allowed: bool,
    pub file_storage_encryption_scheme: String,
    pub file_storage_retention: PossiblyInfinite<Duration<DateLike>>,
    pub mfa_required: bool,
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS>.
    pub tls_profile: TlsProfile,
    pub federation_enabled: bool,
    pub federation_whitelist_enabled: bool,
    pub federation_friendly_servers: LinkedHashSet<String>,
    pub settings_backup_interval: String,
    pub user_data_backup_interval: String,
    pub push_notification_with_body: bool,
    pub push_notification_with_sender: bool,
    pub c2s_unencrypted: bool,
    pub prosody_overrides: Option<ProsodySettings>,
    pub prosody_overrides_raw: Option<Lua>,
}

/// XMPP server configuration, as stored in the database.
///
/// All fields are optional because the Prose Pod API only stores manual overrides.
/// This way, if security defaults are raised, every Prose Pod will automatically benefit from it upon update.
/// Those default values (from [app_config::defaults][crate::app_config::defaults]) can also be overridden
/// by a Prose Pod administrator via the Prose Pod API configuration file (`Prose.toml`).
///
/// When returning the server configuration, the Prose Pod API replaces non-overridden (empty) values
/// by their default. See [ServerConfig] and [ServerConfig::new].
#[serde_as]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[serde_with::skip_serializing_none]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicServerConfig {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub message_archive_enabled: Option<bool>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub message_archive_retention: Option<PossiblyInfinite<Duration<DateLike>>>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub file_upload_allowed: Option<bool>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub file_storage_encryption_scheme: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub file_storage_retention: Option<PossiblyInfinite<Duration<DateLike>>>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub mfa_required: Option<bool>,
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS>.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub tls_profile: Option<TlsProfile>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub federation_enabled: Option<bool>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub federation_whitelist_enabled: Option<bool>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub federation_friendly_servers: Option<LinkedStringSet>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub settings_backup_interval: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub user_data_backup_interval: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub push_notification_with_body: Option<bool>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub push_notification_with_sender: Option<bool>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub c2s_unencrypted: Option<bool>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub prosody_overrides: Option<ProsodySettings>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub prosody_overrides_raw: Option<Lua>,
    // NOTE: Do not use. Only there to support `skip_serializing_none` while
    //   still allowing full read from the key/value store.
    #[serde(default, rename = "domain", skip_serializing)]
    pub(in crate::features::server_config) _domain: Option<JidDomain>,
}

impl ServerConfig {
    pub fn with_default_values(
        dynamic_server_config: &DynamicServerConfig,
        app_config: &AppConfig,
    ) -> ServerConfig {
        let ref static_server_config = app_config.server;

        // NOTE: Destructure so the compiler ensures we will never miss using a field.
        let ServerConfigDefaults {
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
        } = &static_server_config.defaults;

        macro_rules! get_or_default {
            ($var:ident) => {
                dynamic_server_config.$var.unwrap_or($var.to_owned())
            };
            (deref $var:ident) => {
                (dynamic_server_config.$var)
                    .as_deref()
                    .unwrap_or(&$var)
                    .to_owned()
            };
        }

        ServerConfig {
            domain: app_config.server.domain.to_owned(),
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
            c2s_unencrypted: app_config.debug.c2s_unencrypted,
            prosody_overrides: dynamic_server_config.prosody_overrides.to_owned(),
            prosody_overrides_raw: dynamic_server_config.prosody_overrides_raw.to_owned(),
        }
    }
}

/// See <https://wiki.mozilla.org/Security/Server_Side_TLS>.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(strum::EnumIter, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
pub enum TlsProfile {
    /// Modern clients that support TLS 1.3, with no need for backwards compatibility.
    ///
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS#Modern_compatibility>.
    Modern,
    /// Recommended configuration for a general-purpose server.
    ///
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS#Intermediate_compatibility_(recommended)>.
    Intermediate,
    /// Services accessed by very old clients or libraries, such as Internet Explorer 8 (Windows XP), Java 6, or OpenSSL 0.9.8.
    ///
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS#Old_backward_compatibility>.
    Old,
}
