// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod defaults;

use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr as _,
};

use email_address::EmailAddress;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use hickory_resolver::{Name as DomainName, Name as HostName};
use lazy_static::lazy_static;
use linked_hash_set::LinkedHashSet;
pub use prosody_config::ProsodySettings as ProsodyConfig;
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    invitations::InvitationChannel,
    models::{durations::*, xmpp::jid::*, TimeLike, Url},
};

use super::{server_config::TlsProfile, xmpp::JidDomain};

pub use self::pod_config::*;

pub const API_DATA_DIR: &'static str = "/var/lib/prose-pod-api";
pub const API_CONFIG_DIR: &'static str = "/etc/prose";
pub const CONFIG_FILE_NAME: &'static str = "prose.toml";
// NOTE: Hosts are hard-coded here because they're internal to the Prose Pod
//   and cannot be changed via configuration.
pub const ADMIN_HOST: &'static str = "admin.prose.local";
pub const FILE_SHARE_HOST: &'static str = "upload.prose.local";

lazy_static! {
    pub static ref CONFIG_FILE_PATH: PathBuf =
        (Path::new(API_CONFIG_DIR).join(CONFIG_FILE_NAME)).to_path_buf();
}

// TODO: Validate values intervals (e.g. `default_response_timeout`).
/// Prose Pod configuration.
///
/// Structure inspired from [valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)'s
/// [Config](https://github.com/valeriansaliou/vigil/tree/master/src/config).
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub branding: BrandingConfig,
    #[serde(default)]
    pub notifiers: NotifiersConfig,
    #[serde(default)]
    pub log: LogConfig,
    pub pod: PodConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub api: ApiConfig,
    pub dashboard: DashboardConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    /// Advanced config, use only if needed.
    #[serde(default)]
    pub prosody_ext: ProsodyExtConfig,
    /// Advanced config, use only if needed.
    #[serde(default)]
    pub prosody: HashMap<String, ProsodyHostConfig>,
    /// Advanced config, use only if needed.
    #[serde(default)]
    pub bootstrap: BootstrapConfig,
    /// Advanced config, use only if needed.
    #[serde(default)]
    pub service_accounts: ServiceAccountsConfig,
    /// Advanced config, use only if needed.
    #[serde(default, rename = "debug_use_at_your_own_risk")]
    pub debug: DebugConfig,
    /// Advanced config, use only if needed.
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

    pub fn from_figment(mut figment: Figment) -> anyhow::Result<Self> {
        use anyhow::Context as _;
        use figment::providers::*;

        let server_domain = figment.extract_inner::<String>("server.domain")?;

        // NOTE: We have to use `Serialized::default`. See <https://github.com/SergioBenitez/Figment/issues/64#issuecomment-1493111060>.

        // If an email notifier is defined, add a default for the Pod address.
        let smtp_host = figment
            .extract_inner::<String>("notifiers.email.smtp_host")
            .ok();
        if smtp_host.is_some() {
            figment = figment.join(Serialized::default(
                "notifiers.email.pod_address",
                format!("prose@{server_domain}"),
            ));
        }

        let PodAddress {
            domain: pod_domain,
            ipv4: pod_ipv4,
            ipv6: pod_ipv6,
            ..
        } = figment
            .extract_inner::<PodAddress>("pod.address")
            .unwrap_or_default();
        if (pod_ipv4, pod_ipv6) == (None, None) {
            // If no static address has been defined, add a default for the Pod domain.
            let default_server_domain = format!("prose.{server_domain}");
            figment = figment.join(Serialized::default(
                "pod.address.domain",
                &default_server_domain,
            ));

            // If possible, add a default for the Dashboard URL.
            let pod_domain = pod_domain.map_or(default_server_domain, |name| name.to_string());
            figment = figment.join(Serialized::default(
                "dashboard.url",
                format!("https://admin.{pod_domain}"),
            ));
        }

        figment
            .extract()
            .context(format!("Invalid '{CONFIG_FILE_NAME}' configuration file"))
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
    pub fn groups_domain(&self) -> JidDomain {
        use std::str::FromStr as _;

        JidDomain::from_str(&format!("groups.{}", self.server.domain))
            .expect("Domain too long after adding 'groups.' prefix.")
    }

    /// E.g. `your-company.com.`.
    pub fn server_fqdn(&self) -> DomainName {
        self.server_domain().as_fqdn()
    }
    /// E.g. `groups.your-company.com.`.
    pub fn groups_fqdn(&self) -> DomainName {
        self.groups_domain().as_fqdn()
    }
    /// E.g. `prose.your-company.com.` or `your-company.prose.net.`!
    pub fn pod_fqdn(&self) -> DomainName {
        self.pod.network_address().as_fqdn(&self.server_fqdn())
    }
    /// E.g. `prose.your-company.com.`.
    pub fn app_web_fqdn(&self) -> DomainName {
        (HostName::from_str("prose").unwrap())
            .append_domain(&self.server_fqdn())
            .expect("Domain name too long when adding the `prose` prefix")
    }
    /// E.g. `https://prose.your-company.com`.
    pub fn app_web_url(&self) -> Url {
        let mut app_web_fqdn = self.app_web_fqdn();
        app_web_fqdn.set_fqdn(false);
        Url::parse(&format!("https://{app_web_fqdn}"))
            .expect("Cannot make Web app URL from `app_web_fqdn`")
    }
    /// E.g. `admin.prose.your-company.com.`.
    pub fn dashboard_fqdn(&self) -> DomainName {
        (HostName::from_str("admin").unwrap())
            .append_domain(&self.app_web_fqdn())
            .expect("Domain name too long when adding the `admin` prefix")
    }

    pub fn dashboard_url(&self) -> &Url {
        &self.dashboard.url.0
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
pub struct ApiConfig {
    /// IP address to serve on.
    #[serde(default = "defaults::api_address")]
    pub address: IpAddr,
    /// Port to serve on.
    #[serde(default = "defaults::api_port")]
    pub port: u16,
    /// Some requests may take a long time to execute. Sometimes we support
    /// response timeouts, but don't want to hardcode a value.
    #[serde(default = "defaults::api_default_response_timeout")]
    pub default_response_timeout: Duration<TimeLike>,
    #[serde(default = "defaults::api_default_retry_interval")]
    pub default_retry_interval: Duration<TimeLike>,
    #[serde(default)]
    pub databases: DatabasesConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            address: defaults::api_address(),
            port: defaults::api_port(),
            default_response_timeout: defaults::api_default_response_timeout(),
            default_retry_interval: defaults::api_default_retry_interval(),
            databases: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardConfig {
    pub url: DashboardUrl,
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

mod pod_config {
    use hickory_proto::rr::Name as DomainName;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[derive(Debug, Clone)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct PodConfig {
        pub address: PodAddress,
    }

    #[derive(Debug, Clone, Default)]
    #[derive(serdev::Serialize, serdev::Deserialize)]
    #[serde(validate = "Self::validate")]
    pub struct PodAddress {
        pub domain: Option<DomainName>,
        pub ipv4: Option<Ipv4Addr>,
        pub ipv6: Option<Ipv6Addr>,
        /// NOTE: Here to prevent the creation of an invalid value.
        #[serde(skip)]
        _private: (),
    }

    impl PodAddress {
        fn validate(&self) -> Result<(), &'static str> {
            if (&self.domain, &self.ipv4, &self.ipv6) == (&None, &None, &None) {
                return Err("The Pod address must contain at least an IPv4, an IPv6 or a domain.");
            }

            Ok(())
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
    pub defaults: ServerDefaultsConfig,
}

impl ServerConfig {
    pub fn http_url(&self) -> String {
        format!("http://{}:{}", self.local_hostname, self.http_port)
    }
    pub fn admin_http_url(&self) -> String {
        format!("http://{}:{}", self.local_hostname_admin, self.http_port)
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

#[derive(Debug, Clone, Deserialize)]
pub struct ProsodyHostConfig {
    #[serde(default)]
    pub defaults: Option<ProsodyConfig>,
    #[serde(default)]
    pub overrides: Option<ProsodyConfig>,
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
pub struct ServerDefaultsConfig {
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

impl Default for ServerDefaultsConfig {
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
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default = "defaults::branding_api_app_name")]
    pub api_app_name: String,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            company_name: None,
            api_app_name: defaults::branding_api_app_name(),
        }
    }
}

impl Default for InvitationChannel {
    fn default() -> Self {
        defaults::notify_workspace_invitation_channel()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NotifiersConfig {
    #[serde(default = "defaults::notify_workspace_invitation_channel")]
    pub workspace_invitation_channel: InvitationChannel,
    #[serde(default)]
    pub email: Option<EmailNotifierConfig>,
}

impl NotifiersConfig {
    pub fn email<'a>(&'a self) -> Result<&'a EmailNotifierConfig, MissingConfiguration> {
        match self.email {
            Some(ref conf) => Ok(conf),
            None => Err(MissingConfiguration("notifiers.email")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailNotifierConfig {
    pub pod_address: EmailAddress,

    pub smtp_host: String,

    #[serde(default = "defaults::smtp_port")]
    pub smtp_port: u16,

    pub smtp_username: Option<String>,
    pub smtp_password: Option<SecretString>,

    #[serde(default = "defaults::smtp_encrypt")]
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
    // NOTE: Needs to be available in release builds so we can run the CI in
    //   `prose-pod-system` without having to start live notifiers.
    #[serde(default)]
    pub skip_startup_actions: HashSet<String>,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_config_at_startup: defaults::true_in_debug(),
            detailed_error_responses: defaults::true_in_debug(),
            c2s_unencrypted: defaults::always_false(),
            skip_startup_actions: Default::default(),
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
    "Missing key `{0}` the app configuration. Add it to `prose.toml` or use environment variables."
)]
pub struct MissingConfiguration(pub &'static str);

// MARK: Dashboard URL

#[derive(Debug, Clone)]
#[derive(serdev::Serialize, serdev::Deserialize)]
#[serde(validate = "Self::validate")]
pub struct DashboardUrl(Url);

impl DashboardUrl {
    pub fn new(url: Url) -> anyhow::Result<Self> {
        let res = Self(url);
        res.validate().map_err(|str| anyhow::Error::msg(str))?;
        Ok(res)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if url_has_no_path(&self.0) {
            Ok(())
        } else {
            Err("The Dashboard URL contains a fragment or query.")
        }
    }
}

impl std::ops::Deref for DashboardUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// MARK: - Helpers

fn url_has_no_path(url: &Url) -> bool {
    // NOTE: `make_relative` when called on the same URL returns only the fragment and query.
    let relative_part = url.make_relative(&url);
    relative_part.is_some_and(|s| s.is_empty())
}
