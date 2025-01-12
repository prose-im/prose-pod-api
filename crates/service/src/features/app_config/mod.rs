// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod defaults;

use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr as _,
};

use email_address::EmailAddress;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use secrecy::SecretString;
use serde::Deserialize;
use url_serde::SerdeUrl;

use crate::{
    invitations::InvitationChannel,
    models::{
        durations::{DateLike, Duration, PossiblyInfinite},
        xmpp::jid::{BareJid, DomainPart, DomainRef, JidNode},
        TimeLike,
    },
};

use super::server_config::TlsProfile;

pub const CONFIG_FILE_NAME: &'static str = "Prose.toml";
// NOTE: Hosts are hard-coded here because they're internal to the Prose Pod
//   and cannot be changed via configuration.
pub const ADMIN_HOST: &'static str = "admin.prose.org.local";
pub const FILE_SHARE_HOST: &'static str = "upload.prose.org.local";

pub type Config = AppConfig;

/// Prose Pod configuration.
///
/// Structure inspired from [valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)'s
/// [Config](https://github.com/valeriansaliou/vigil/tree/master/src/config).
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub service_accounts: ConfigServiceAccounts,
    #[serde(default)]
    pub bootstrap: ConfigBootstrap,
    #[serde(default)]
    pub server: ConfigServer,
    pub branding: ConfigBranding,
    #[serde(default)]
    pub notify: ConfigNotify,
    #[serde(default)]
    pub databases: ConfigDatabases,
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
    #[cfg(debug_assertions)]
    #[serde(default)]
    pub debug_only: ConfigDebugOnly,
}

impl AppConfig {
    pub fn figment() -> Figment {
        Self::figment_at_path(CONFIG_FILE_NAME)
    }

    pub fn figment_at_path(path: impl AsRef<Path>) -> Figment {
        // NOTE: See what's possible at <https://docs.rs/figment/latest/figment/>.
        Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("PROSE_").split("__"))
    }

    pub fn from_figment(figment: Figment) -> Self {
        figment
            .extract()
            .unwrap_or_else(|e| panic!("Invalid '{CONFIG_FILE_NAME}' configuration file: {e}"))
        // TODO: Check values intervals (e.g. `default_response_timeout`).
    }

    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self::from_figment(Self::figment_at_path(path))
    }

    pub fn from_default_figment() -> Self {
        Self::from_figment(Self::figment())
    }

    pub fn api_jid(&self) -> BareJid {
        BareJid::from_parts(
            Some(&self.service_accounts.prose_pod_api.xmpp_node),
            &DomainPart::from_str(ADMIN_HOST).unwrap(),
        )
    }

    pub fn workspace_jid(&self, domain: &DomainRef) -> BareJid {
        BareJid::from_parts(
            Some(&self.service_accounts.prose_workspace.xmpp_node),
            domain,
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServiceAccounts {
    #[serde(default = "defaults::service_accounts_prose_pod_api")]
    pub prose_pod_api: ConfigServiceAccount,
    #[serde(default = "defaults::service_accounts_prose_workspace")]
    pub prose_workspace: ConfigServiceAccount,
}

impl Default for ConfigServiceAccounts {
    fn default() -> Self {
        Self {
            prose_pod_api: defaults::service_accounts_prose_pod_api(),
            prose_workspace: defaults::service_accounts_prose_workspace(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServiceAccount {
    pub xmpp_node: JidNode,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigBootstrap {
    pub prose_pod_api_xmpp_password: Option<SecretString>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServer {
    #[serde(default = "defaults::server_local_hostname")]
    pub local_hostname: String,
    #[serde(default = "defaults::server_local_hostname_admin")]
    pub local_hostname_admin: String,
    #[serde(default = "defaults::server_http_port")]
    pub http_port: u16,
    #[serde(default = "defaults::server_prosody_config_file_path")]
    pub prosody_config_file_path: PathBuf,
    #[serde(default = "defaults::server_oauth2_registration_key")]
    pub oauth2_registration_key: SecretString,
    #[serde(default)]
    pub defaults: ConfigServerDefaults,
}

impl ConfigServer {
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

impl Default for ConfigServer {
    fn default() -> Self {
        Self {
            local_hostname: defaults::server_local_hostname(),
            local_hostname_admin: defaults::server_local_hostname_admin(),
            http_port: defaults::server_http_port(),
            prosody_config_file_path: defaults::server_prosody_config_file_path(),
            oauth2_registration_key: defaults::server_oauth2_registration_key(),
            defaults: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServerDefaults {
    pub message_archive_enabled: bool,
    pub message_archive_retention: PossiblyInfinite<Duration<DateLike>>,
    pub file_upload_allowed: bool,
    pub file_storage_encryption_scheme: String,
    pub file_storage_retention: PossiblyInfinite<Duration<DateLike>>,
    pub mfa_required: bool,
    pub tls_profile: TlsProfile,
    pub federation_enabled: bool,
    pub settings_backup_interval: String,
    pub user_data_backup_interval: String,
    pub push_notification_with_body: bool,
    pub push_notification_with_sender: bool,
}

impl Default for ConfigServerDefaults {
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
            settings_backup_interval: defaults::server_defaults_settings_backup_interval(),
            user_data_backup_interval: defaults::server_defaults_user_data_backup_interval(),
            push_notification_with_body: defaults::server_defaults_push_notification_with_body(),
            push_notification_with_sender: defaults::server_defaults_push_notification_with_sender(
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigBranding {
    #[serde(default = "defaults::branding_page_title")]
    pub page_title: String,
    pub page_url: SerdeUrl,
    pub company_name: String,
}

impl Default for InvitationChannel {
    fn default() -> Self {
        defaults::notify_workspace_invitation_channel()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigNotify {
    #[serde(default = "defaults::notify_workspace_invitation_channel")]
    pub workspace_invitation_channel: InvitationChannel,
    #[serde(default)]
    pub email: Option<ConfigNotifyEmail>,
}

impl ConfigNotify {
    pub fn email<'a>(&'a self) -> Result<&'a ConfigNotifyEmail, MissingConfiguration> {
        match self.email {
            Some(ref conf) => Ok(conf),
            None => Err(MissingConfiguration("notify.email")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigNotifyEmail {
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
pub struct ConfigDatabases {
    #[serde(default = "defaults::databases_main")]
    pub main: ConfigDatabase,
}

impl Default for ConfigDatabases {
    fn default() -> Self {
        Self {
            main: defaults::databases_main(),
        }
    }
}

/// Inspired by <https://github.com/SeaQL/sea-orm/blob/bead32a0d812fd9c80c57e91e956e9d90159e067/sea-orm-rocket/lib/src/config.rs>.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigDatabase {
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

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigDebugOnly {
    #[serde(default)]
    pub automatically_accept_invitations: bool,
    /// When automatically accepting invitations during testing, one might want to authenticate
    /// the created member. With this flag turned on, the member's password will be their JID.
    #[serde(default)]
    pub insecure_password_on_auto_accept_invitation: bool,
    #[serde(default)]
    pub dependency_modes: ConfigDependencyModes,
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
pub struct ConfigDependencyModes {
    #[serde(default)]
    pub uuid: UuidDependencyMode,
    #[serde(default)]
    pub notifier: NotifierDependencyMode,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Missing key `{0}` the app configuration. Add it to `Prose.toml` or use environment variables."
)]
pub struct MissingConfiguration(&'static str);
