// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod defaults;

#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr as _,
};

use anyhow::Context;
use email_address::EmailAddress;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use lazy_static::lazy_static;
use linked_hash_set::LinkedHashSet;
pub use prosody_config::ProsodySettings as ProsodyConfig;
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    invitations::InvitationChannel,
    models::{
        durations::{DateLike, Duration, PossiblyInfinite},
        xmpp::jid::{BareJid, DomainPart, JidNode},
        TimeLike,
    },
};

pub use super::pod_config::PodConfig;
use super::{server_config::TlsProfile, xmpp::JidDomain};

pub const API_DATA_DIR: &'static str = "/var/lib/prose-pod-api";
pub const API_CONFIG_DIR: &'static str = "/etc/prose-pod-api";
pub const CONFIG_FILE_NAME: &'static str = "Prose.toml";
// NOTE: Hosts are hard-coded here because they're internal to the Prose Pod
//   and cannot be changed via configuration.
pub const ADMIN_HOST: &'static str = "admin.prose.org.local";
pub const FILE_SHARE_HOST: &'static str = "upload.prose.org.local";

lazy_static! {
    pub static ref CONFIG_FILE_PATH: PathBuf =
        (Path::new(API_CONFIG_DIR).join(CONFIG_FILE_NAME)).to_path_buf();
}

pub type Config = AppConfig;

/// Prose Pod configuration.
///
/// Structure inspired from [valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)'s
/// [Config](https://github.com/valeriansaliou/vigil/tree/master/src/config).
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub service_accounts: ServiceAccountsConfig,
    #[serde(default)]
    pub bootstrap: BootstrapConfig,
    pub server: ServerConfig,
    pub pod: PodConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub prosody_ext: ProsodyExtConfig,
    #[serde(default)]
    pub prosody: ProsodyConfig,
    #[serde(default)]
    pub branding: BrandingConfig,
    #[serde(default)]
    pub notify: NotifyConfig,
    #[serde(default)]
    pub databases: DatabasesConfig,
    /// IP address to serve on.
    #[serde(default = "defaults::address")]
    pub address: IpAddr,
    /// Port to serve on.
    #[serde(default = "defaults::port")]
    pub port: u16,
    /// Some requests may take a long time to execute. Sometimes we support
    /// response timeouts, but don't want to hardcode a value.
    #[serde(default = "defaults::default_response_timeout")]
    pub default_response_timeout: Duration<TimeLike>,
    #[serde(default = "defaults::default_retry_interval")]
    pub default_retry_interval: Duration<TimeLike>,
    #[serde(default, rename = "debug_use_at_your_own_risk")]
    pub debug: DebugConfig,
    #[cfg(debug_assertions)]
    #[serde(default)]
    pub debug_only: DebugOnlyConfig,
}

impl AppConfig {
    pub fn figment() -> Figment {
        Self::figment_at_path(CONFIG_FILE_PATH.as_path())
    }

    pub fn figment_at_path(path: impl AsRef<Path>) -> Figment {
        // NOTE: See what's possible at <https://docs.rs/figment/latest/figment/>.
        Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("PROSE_").split("__"))
    }

    pub fn from_figment(figment: Figment) -> anyhow::Result<Self> {
        figment
            .extract()
            .context("Invalid '{CONFIG_FILE_NAME}' configuration file")
        // TODO: Check values intervals (e.g. `default_response_timeout`).
    }

    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::from_figment(Self::figment_at_path(path))
    }

    pub fn from_default_figment() -> anyhow::Result<Self> {
        Self::from_figment(Self::figment())
    }

    pub fn api_jid(&self) -> BareJid {
        BareJid::from_parts(
            Some(&self.service_accounts.prose_pod_api.xmpp_node),
            &DomainPart::from_str(ADMIN_HOST).unwrap(),
        )
    }

    pub fn workspace_jid(&self) -> BareJid {
        BareJid::from_parts(
            Some(&self.service_accounts.prose_workspace.xmpp_node),
            &self.server.domain,
        )
    }

    pub fn server_domain(&self) -> &JidDomain {
        &self.server.domain
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default = "defaults::log_level")]
    pub level: LogLevel,
    #[serde(default = "defaults::log_format")]
    pub format: LogFormat,
    #[serde(default = "defaults::log_timer")]
    pub timer: LogTimer,
    #[serde(default = "defaults::true_in_debug")]
    pub with_ansi: bool,
    #[serde(default = "defaults::true_in_debug")]
    pub with_file: bool,
    #[serde(default = "defaults::always_true")]
    pub with_level: bool,
    #[serde(default = "defaults::always_true")]
    pub with_target: bool,
    #[serde(default = "defaults::always_false")]
    pub with_thread_ids: bool,
    #[serde(default = "defaults::true_in_debug")]
    pub with_line_number: bool,
    #[serde(default = "defaults::always_false")]
    pub with_span_events: bool,
    #[serde(default = "defaults::true_in_debug")]
    pub with_thread_names: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: defaults::log_level(),
            format: defaults::log_format(),
            timer: defaults::log_timer(),
            with_ansi: defaults::true_in_debug(),
            with_file: defaults::true_in_debug(),
            with_level: defaults::always_true(),
            with_target: defaults::always_true(),
            with_thread_ids: defaults::always_false(),
            with_line_number: defaults::always_true(),
            with_span_events: defaults::always_false(),
            with_thread_names: defaults::true_in_debug(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
#[derive(strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
#[derive(strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LogFormat {
    Full,
    Compact,
    Json,
    Pretty,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
#[derive(strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LogTimer {
    None,
    Time,
    Uptime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountsConfig {
    #[serde(default = "defaults::service_accounts_prose_pod_api")]
    pub prose_pod_api: ServiceAccountConfig,
    #[serde(default = "defaults::service_accounts_prose_workspace")]
    pub prose_workspace: ServiceAccountConfig,
}

impl Default for ServiceAccountsConfig {
    fn default() -> Self {
        Self {
            prose_pod_api: defaults::service_accounts_prose_pod_api(),
            prose_workspace: defaults::service_accounts_prose_workspace(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountConfig {
    pub xmpp_node: JidNode,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BootstrapConfig {
    #[serde(default = "defaults::bootstrap_prose_pod_api_xmpp_password")]
    pub prose_pod_api_xmpp_password: SecretString,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            prose_pod_api_xmpp_password: defaults::bootstrap_prose_pod_api_xmpp_password(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub domain: JidDomain,
    #[serde(default = "defaults::server_local_hostname")]
    pub local_hostname: String,
    #[serde(default = "defaults::server_local_hostname_admin")]
    pub local_hostname_admin: String,
    #[serde(default = "defaults::server_http_port")]
    pub http_port: u16,
    #[serde(default = "defaults::server_log_level")]
    pub log_level: prosody_config::LogLevel,
    #[serde(default)]
    pub defaults: ServerConfigDefaults,
}

impl ServerConfig {
    pub fn oauth2_api_url(&self) -> String {
        format!("http://{}:{}/oauth2", self.local_hostname, self.http_port)
    }
    pub fn rest_api_url(&self) -> String {
        format!("http://{}:{}/rest", self.local_hostname, self.http_port)
    }
    pub fn admin_rest_api_url(&self) -> String {
        format!(
            "http://{}:{}/admin_rest",
            self.local_hostname_admin, self.http_port
        )
    }
    pub fn admin_rest_api_on_main_host_url(&self) -> String {
        format!(
            "http://{}:{}/admin_rest",
            self.local_hostname, self.http_port
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "defaults::auth_token_ttl")]
    pub token_ttl: iso8601_duration::Duration,
    #[serde(default = "defaults::auth_password_reset_token_ttl")]
    pub password_reset_token_ttl: iso8601_duration::Duration,
    #[serde(default = "defaults::auth_oauth2_registration_key")]
    pub oauth2_registration_key: SecretString,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_ttl: defaults::auth_token_ttl(),
            password_reset_token_ttl: defaults::auth_password_reset_token_ttl(),
            oauth2_registration_key: defaults::auth_oauth2_registration_key(),
        }
    }
}

/// NOTE: We cannot include [`ProsodySettings`] as a flattened field because
///   `#[serde(deny_unknown_fields)]` doesn’t work with `#[serde(flatten)]`.
///   See <https://serde.rs/container-attrs.html#deny_unknown_fields>.
#[derive(Debug, Clone, Deserialize)]
pub struct ProsodyExtConfig {
    #[serde(default = "defaults::prosody_config_file_path")]
    pub config_file_path: PathBuf,
    /// NOTE: Those modules will be added to `modules_enabled` after everything
    ///   else has been applied (apart from dynamic overrides, which are always
    ///   applied last).
    #[serde(default)]
    pub additional_modules_enabled: Vec<String>,
}

impl Default for ProsodyExtConfig {
    fn default() -> Self {
        Self {
            config_file_path: defaults::prosody_config_file_path(),
            additional_modules_enabled: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfigDefaults {
    pub message_archive_enabled: bool,
    pub message_archive_retention: PossiblyInfinite<Duration<DateLike>>,
    pub file_upload_allowed: bool,
    pub file_storage_encryption_scheme: String,
    pub file_storage_retention: PossiblyInfinite<Duration<DateLike>>,
    pub mfa_required: bool,
    pub tls_profile: TlsProfile,
    pub federation_enabled: bool,
    pub federation_whitelist_enabled: bool,
    pub federation_friendly_servers: LinkedHashSet<String>,
    pub settings_backup_interval: String,
    pub user_data_backup_interval: String,
    pub push_notification_with_body: bool,
    pub push_notification_with_sender: bool,
}

impl Default for ServerConfigDefaults {
    fn default() -> Self {
        Self {
            message_archive_enabled: defaults::server_defaults_message_archive_enabled(),
            message_archive_retention: defaults::server_defaults_message_archive_retention(),
            file_upload_allowed: defaults::server_defaults_file_upload_allowed(),
            file_storage_encryption_scheme:
                defaults::server_defaults_file_storage_encryption_scheme(),
            file_storage_retention: defaults::server_defaults_file_storage_retention(),
            mfa_required: defaults::server_defaults_mfa_required(),
            tls_profile: defaults::server_defaults_tls_profile(),
            federation_enabled: defaults::server_defaults_federation_enabled(),
            federation_whitelist_enabled: defaults::server_defaults_federation_whitelist_enabled(),
            federation_friendly_servers: defaults::server_defaults_federation_friendly_servers(),
            settings_backup_interval: defaults::server_defaults_settings_backup_interval(),
            user_data_backup_interval: defaults::server_defaults_user_data_backup_interval(),
            push_notification_with_body: defaults::server_defaults_push_notification_with_body(),
            push_notification_with_sender: defaults::server_defaults_push_notification_with_sender(
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrandingConfig {
    #[serde(default = "defaults::branding_page_title")]
    pub page_title: String,
    #[serde(default)]
    pub company_name: Option<String>,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            page_title: defaults::branding_page_title(),
            company_name: None,
        }
    }
}

impl Default for InvitationChannel {
    fn default() -> Self {
        defaults::notify_workspace_invitation_channel()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NotifyConfig {
    #[serde(default = "defaults::notify_workspace_invitation_channel")]
    pub workspace_invitation_channel: InvitationChannel,
    #[serde(default)]
    pub email: Option<NotifyEmailConfig>,
}

impl NotifyConfig {
    pub fn email<'a>(&'a self) -> Result<&'a NotifyEmailConfig, MissingConfiguration> {
        match self.email {
            Some(ref conf) => Ok(conf),
            None => Err(MissingConfiguration("notify.email")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotifyEmailConfig {
    pub pod_address: EmailAddress,

    #[serde(default = "defaults::notify_email_smtp_host")]
    pub smtp_host: String,

    #[serde(default = "defaults::notify_email_smtp_port")]
    pub smtp_port: u16,

    pub smtp_username: Option<String>,
    pub smtp_password: Option<SecretString>,

    #[serde(default = "defaults::notify_email_smtp_encrypt")]
    pub smtp_encrypt: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabasesConfig {
    #[serde(default = "defaults::databases_main")]
    pub main: DatabaseConfig,
}

impl Default for DatabasesConfig {
    fn default() -> Self {
        Self {
            main: defaults::databases_main(),
        }
    }
}

/// Inspired by <https://github.com/SeaQL/sea-orm/blob/bead32a0d812fd9c80c57e91e956e9d90159e067/sea-orm-rocket/lib/src/config.rs>.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default)]
    pub min_connections: Option<u32>,
    #[serde(default = "defaults::database_max_connections")]
    pub max_connections: usize,
    #[serde(default = "defaults::database_connect_timeout")]
    pub connect_timeout: u64,
    #[serde(default)]
    pub idle_timeout: Option<u64>,
    #[serde(default)]
    pub sqlx_logging: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebugConfig {
    #[serde(default = "defaults::true_in_debug")]
    pub log_config_at_startup: bool,
    #[serde(default = "defaults::true_in_debug")]
    pub detailed_error_responses: bool,
    #[serde(default = "defaults::always_false")]
    pub c2s_unencrypted: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_config_at_startup: defaults::true_in_debug(),
            detailed_error_responses: defaults::true_in_debug(),
            c2s_unencrypted: defaults::always_false(),
        }
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DebugOnlyConfig {
    /// When automatically accepting invitations during testing, one might want to authenticate
    /// the created member. With this flag turned on, the member's password will be their JID.
    #[serde(default)]
    pub insecure_password_on_auto_accept_invitation: bool,
    #[serde(default)]
    pub dependency_modes: DependencyModesConfig,
    #[serde(default)]
    pub skip_startup_actions: HashSet<String>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UuidDependencyMode {
    Normal,
    Incrementing,
}

#[cfg(debug_assertions)]
impl Default for UuidDependencyMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifierDependencyMode {
    Live,
    Logging,
}

#[cfg(debug_assertions)]
impl Default for NotifierDependencyMode {
    fn default() -> Self {
        Self::Live
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DependencyModesConfig {
    #[serde(default)]
    pub uuid: UuidDependencyMode,
    #[serde(default)]
    pub notifier: NotifierDependencyMode,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Missing key `{0}` the app configuration. Add it to `Prose.toml` or use environment variables."
)]
pub struct MissingConfiguration(pub &'static str);
