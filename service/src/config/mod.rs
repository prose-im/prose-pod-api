// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod defaults;

use std::{path::PathBuf, str::FromStr as _};

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use jid::DomainPart;
use prose_xmpp::BareJid;
use secrecy::SecretString;
use serde::Deserialize;
use url_serde::SerdeUrl;

use crate::model::{
    DateLike, Duration, InvitationChannel, JidNode, PossiblyInfinite, ServerConfig,
};

// NOTE: Hosts are hard-coded here because they're internal to the Prose Pod
//   and cannot be changed via configuration.
pub const ADMIN_HOST: &'static str = "admin.prose.org.local";
pub const FILE_SHARE_HOST: &'static str = "upload.prose.org.local";

pub type AppConfig = Config;

/// Prose Pod configuration.
///
/// Structure inspired from [valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)'s
/// [Config](https://github.com/valeriansaliou/vigil/tree/master/src/config).
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub service_accounts: ConfigServiceAccounts,
    #[serde(default)]
    pub bootstrap: ConfigBootstrap,
    #[serde(default)]
    pub server: ConfigServer,
    pub branding: ConfigBranding,
    #[serde(default)]
    pub notify: ConfigNotify,
    #[cfg(debug_assertions)]
    #[serde(default)]
    pub debug_only: ConfigDebugOnly,
}

impl Config {
    pub fn figment() -> Self {
        // NOTE: See what's possible at <https://docs.rs/figment/latest/figment/>.
        Figment::new()
            .merge(Toml::file("Prose.toml"))
            .merge(Env::prefixed("PROSE_").split("__"))
            .extract()
            .expect("Could not read config")
    }

    pub fn api_jid(&self) -> BareJid {
        BareJid::from_parts(
            Some(&self.service_accounts.prose_pod_api.xmpp_node),
            &DomainPart::from_str(ADMIN_HOST).unwrap(),
        )
    }

    pub fn workspace_jid(&self, server_config: &ServerConfig) -> BareJid {
        BareJid::from_parts(
            Some(&self.service_accounts.prose_workspace.xmpp_node),
            &DomainPart::from_str(&server_config.domain).unwrap(),
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
    pub minimum_tls_version: String,
    pub minimum_cipher_suite: String,
    pub federation_enabled: bool,
    pub settings_backup_interval: String,
    pub user_data_backup_interval: String,
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
            minimum_tls_version: defaults::server_defaults_minimum_tls_version(),
            minimum_cipher_suite: defaults::server_defaults_minimum_cipher_suite(),
            federation_enabled: defaults::server_defaults_federation_enabled(),
            settings_backup_interval: defaults::server_defaults_settings_backup_interval(),
            user_data_backup_interval: defaults::server_defaults_user_data_backup_interval(),
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

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigNotifyEmail {
    pub to: String,
    pub from: String,

    #[serde(default = "defaults::notify_email_smtp_host")]
    pub smtp_host: String,

    #[serde(default = "defaults::notify_email_smtp_port")]
    pub smtp_port: u16,

    pub smtp_username: Option<String>,
    pub smtp_password: Option<SecretString>,

    #[serde(default = "defaults::notify_email_smtp_encrypt")]
    pub smtp_encrypt: bool,
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
