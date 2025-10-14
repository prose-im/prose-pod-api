// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use figment::Figment;
use hickory_resolver::Name as DomainName;
use lazy_static::lazy_static;
use validator::Validate;

pub use self::api::*;
pub use self::auth::*;
pub use self::bootstrap::*;
pub use self::branding::*;
pub use self::dashboard::*;
pub use self::debug::*;
#[cfg(debug_assertions)]
pub use self::debug_only::*;
pub use self::log::*;
pub use self::notifiers::*;
pub use self::pod::*;
pub use self::prosody::*;
pub use self::prosody_ext::*;
pub use self::public_contacts::*;
pub use self::server::*;
pub use self::server_api::*;
pub use self::service_accounts::*;

pub const API_DATA_DIR: &'static str = "/var/lib/prose-pod-api";
pub const API_CONFIG_DIR: &'static str = "/etc/prose";
pub const CONFIG_FILE_NAME: &'static str = "prose.toml";
pub const FILE_SHARE_HOST: &'static str = "upload.prose.local";

lazy_static! {
    pub static ref CONFIG_FILE_PATH: PathBuf =
        (Path::new(API_CONFIG_DIR).join(CONFIG_FILE_NAME)).to_path_buf();
    static ref DEFAULT_MAIN_DATABASE_URL: String =
        format!("sqlite://{API_DATA_DIR}/database.sqlite");
    static ref DEFAULT_DB_MAX_READ_CONNECTIONS: usize = {
        let workers: usize =
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        workers * 4
    };
}

mod prelude {
    pub use std::borrow::Cow;
    pub use std::collections::HashSet;
    pub use std::path::PathBuf;

    pub use linked_hash_set::LinkedHashSet;
    pub use secrecy::SecretString;
    pub use serdev::Serialize;
    pub use validator::{Validate, ValidationError, ValidationErrors};

    pub use crate::{errors::MissingConfiguration, models::*};

    pub use super::{util::*, AppConfig};
}

pub mod pub_defaults {
    pub const API_ADDRESS: std::net::IpAddr = std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED);

    pub const API_PORT: u16 = 8080;

    pub const SERVER_HTTP_PORT: u16 = 5280;

    pub const SERVER_API_PORT: u16 = 8080;

    pub const SERVER_LOCAL_HOSTNAME: &'static str = "prose-pod-server";
}

// TODO: Remove default server values from here and use the ones defined in
//   `prose-pod-server` to avoid discrepancies.
fn default_config_static() -> Figment {
    use self::pub_defaults::*;
    use figment::providers::{Format as _, Toml};
    use toml::toml;

    let default_database_url = DEFAULT_MAIN_DATABASE_URL.as_str();

    let default_log_format = if cfg!(debug_assertions) {
        "pretty"
    } else {
        "json"
    };
    let default_log_timer = if cfg!(debug_assertions) {
        "uptime"
    } else {
        "time"
    };

    let true_in_debug = cfg!(debug_assertions);

    let api_address = API_ADDRESS.to_string();

    let static_defaults = toml! {
        [branding]
        api_app_name = "Prose Pod API"

        [log]
        level = "info"
        format = default_log_format
        timer = default_log_timer
        with_ansi = true_in_debug
        with_file = true_in_debug
        with_level = true
        with_target = true
        with_thread_ids = false
        with_line_number = true_in_debug
        with_span_events = false
        with_thread_names = true_in_debug

        [api]
        address = api_address
        port = API_PORT

        [notifiers]
        workspace_invitation_channel = "email"

        [api.databases.main]
        url = default_database_url
        connect_timeout = 5
        sqlx_logging = false

        [auth]
        token_ttl = "PT3H"
        password_reset_token_ttl = "PT15M"
        invitation_ttl = "P1W"

        [api.network_checks]
        timeout = "PT5M"
        retry_interval = "PT5S"
        retry_timeout = "PT1S"
        // When querying DNS records, we query the authoritative name servers directly.
        // To avoid unnecessary DNS queries, we cache the IP addresses of these servers.
        // However, these IP addresses can change over time so we need to clear the cache
        // every now and then. 5 minutes seems long enough to avoid unnecessary queries
        // while a user is checking their DNS configuration, but short enough to react to
        // DNS server reconfigurations.
        dns_cache_ttl = "PT5M"

        [api.member_enriching]
        // When enriching members, we query the XMPP server for all vCards. To
        // avoid flooding the server with too many requests, we cache enriched
        // members for a little while (enough for someone to finish searching for a
        // member, but short enough to react to changes). Enriching isn’t a very
        // costly operation but we wouldn’t want to enrich all members for every
        // keystroke in the search bar of the Dashboard.
        cache_ttl = "PT2M"

        [api.invitations]
        invitation_ttl = "P3D"

        [server]
        local_hostname = SERVER_LOCAL_HOSTNAME
        http_port = SERVER_HTTP_PORT
        log_level = "info"

        [server_api]
        port = SERVER_API_PORT

        [server.defaults]
        message_archive_enabled = true
        message_archive_retention = "infinite"
        file_upload_allowed = true
        file_storage_retention = "infinite"
        file_storage_encryption_scheme = "AES-256"
        mfa_required = true
        tls_profile = "modern"
        federation_enabled = false
        // Federate with the whole XMPP network by default.
        federation_whitelist_enabled = false
        // Do not trust any other server by default
        // (useless until whitelist enabled).
        federation_friendly_servers = []
        settings_backup_interval = "P1D"
        user_data_backup_interval = "P1W"
        push_notification_with_body = true
        push_notification_with_sender = true

        [service_accounts.prose_workspace]
        xmpp_node = "prose-workspace"

        [prosody_ext]
        config_file_path = "/etc/prosody/prosody.cfg.lua"

        [debug_use_at_your_own_risk]
        log_config_at_startup = true_in_debug
        detailed_error_responses = true_in_debug
        c2s_unencrypted = false
    }
    .to_string();

    Figment::from(Toml::string(&static_defaults))
}

fn with_dynamic_defaults(mut figment: Figment) -> anyhow::Result<Figment> {
    use figment::{providers::*, value::Value};

    let server_domain = figment.extract_inner::<String>("server.domain")?;

    // NOTE: We have to use `Serialized::default`. See <https://github.com/SergioBenitez/Figment/issues/64#issuecomment-1493111060>.

    if figment.contains("notifiers.email") {
        figment = figment
            .join(Serialized::default("notifiers.email.smtp_port", 587))
            .join(Serialized::default("notifiers.email.smtp_encrypt", true));
    }

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

    if let Ok(main_default) = figment.extract_inner::<Value>("api.databases.main") {
        // Use `main` as default for `main_read` and `main_write`.
        figment = figment
            .join(Serialized::default(
                "api.databases.main_read",
                main_default.clone(),
            ))
            .join(Serialized::default(
                "api.databases.main_write",
                main_default,
            ));

        // Override default `max_connections` for `main_write`.
        figment = figment
            .join(Serialized::default(
                "api.databases.main_read.max_connections",
                *DEFAULT_DB_MAX_READ_CONNECTIONS,
            ))
            .join(Serialized::default(
                "api.databases.main_write.max_connections",
                1,
            ));
    }

    Ok(figment)
}

impl AppConfig {
    pub fn figment() -> Figment {
        Self::figment_at_path(CONFIG_FILE_PATH.as_path())
    }

    pub fn figment_at_path(path: impl AsRef<Path>) -> Figment {
        use figment::providers::{Env, Format, Toml};

        // NOTE: See what's possible at <https://docs.rs/figment/latest/figment/>.
        default_config_static()
            .merge(Toml::file(path))
            .merge(Env::prefixed("PROSE_").split("__"))
    }

    pub fn from_figment(figment: Figment) -> anyhow::Result<Self> {
        use anyhow::Context as _;

        with_dynamic_defaults(figment)?
            .extract()
            .context(format!("Invalid '{CONFIG_FILE_NAME}' configuration file"))
    }

    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::from_figment(Self::figment_at_path(path))
    }

    pub fn from_default_figment() -> anyhow::Result<Self> {
        Self::from_figment(Self::figment())
    }
}

// MARK: - Data structures

// TODO: Validate values intervals.
/// Prose Pod configuration.
///
/// Structure inspired from [valeriansaliou/vigil](https://github.com/valeriansaliou/vigil)'s
/// [Config](https://github.com/valeriansaliou/vigil/tree/master/src/config).
#[derive(Debug)]
#[derive(Validate, serdev::Deserialize)]
// NOTE: During development, we often have `PROSE_`-prefixed environment
//   variables defined (e.g. `PROSE_POD_API_DIR`), which makes serde complain
//   “unknown field: found `pod_api_dir`”. Enabling `deny_unknown_fields` only
//   in release mode should work around it.
#[cfg_attr(not(debug_assertions), serde(deny_unknown_fields))]
#[validate(nest_all_fields)]
#[serde(validate = "Validate::validate")]
pub struct AppConfig {
    pub branding: Arc<BrandingConfig>,

    pub notifiers: Arc<NotifiersConfig>,

    pub log: Arc<LogConfig>,

    pub pod: Arc<PodConfig>,

    pub server: Arc<ServerConfig>,

    pub server_api: Arc<ServerApiConfig>,

    pub api: Arc<ApiConfig>,

    pub dashboard: Arc<DashboardConfig>,

    pub auth: Arc<AuthConfig>,

    #[serde(default)]
    pub public_contacts: Arc<PublicContactsConfig>,

    /// Advanced config, use only if needed.
    pub prosody_ext: Arc<ProsodyExtConfig>,

    /// Advanced config, use only if needed.
    #[serde(default)]
    pub prosody: HashMap<DomainName, ProsodyHostConfig>,

    /// Advanced config, use only if needed.
    #[serde(default)]
    pub bootstrap: Arc<BootstrapConfig>,

    /// Advanced config, use only if needed.
    pub service_accounts: Arc<ServiceAccountsConfig>,

    /// Advanced config, use only if needed.
    #[serde(rename = "debug_use_at_your_own_risk")]
    pub debug: Arc<DebugConfig>,

    /// Advanced config, use only if needed.
    #[cfg(debug_assertions)]
    #[serde(default)]
    pub debug_only: Arc<DebugOnlyConfig>,
}

mod log {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct LogConfig {
        pub level: LogLevel,

        pub format: LogFormat,

        pub timer: LogTimer,

        pub with_ansi: bool,

        pub with_file: bool,

        pub with_level: bool,

        pub with_target: bool,

        pub with_thread_ids: bool,

        pub with_line_number: bool,

        pub with_span_events: bool,

        pub with_thread_names: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
    #[derive(strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    pub enum LogFormat {
        Full,
        Compact,
        Json,
        Pretty,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
    #[derive(strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    pub enum LogTimer {
        None,
        Time,
        Uptime,
    }
}

mod api {
    use std::net::IpAddr;

    use serdev::Deserialize;

    use super::prelude::*;

    pub use self::databases::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ApiConfig {
        /// IP address to serve on.
        #[validate(skip)]
        pub address: IpAddr,

        /// Port to serve on.
        #[validate(skip)]
        pub port: u16,

        #[validate(nested)]
        pub databases: DatabasesConfig,

        // TODO: Validate.
        pub network_checks: NetworkChecksConfig,

        // TODO: Validate.
        pub member_enriching: MemberEnrichingConfig,

        // TODO: Validate.
        pub invitations: InvitationsConfig,
    }

    // TODO: Validate values.
    #[derive(Debug, Clone, Copy)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct NetworkChecksConfig {
        pub timeout: Duration<TimeLike>,

        pub retry_interval: Duration<TimeLike>,

        pub retry_timeout: Duration<TimeLike>,

        pub dns_cache_ttl: Duration<TimeLike>,
    }

    // TODO: Validate values.
    #[derive(Debug, Clone, Copy)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MemberEnrichingConfig {
        pub cache_ttl: Duration<TimeLike>,
    }

    // TODO: Validate values.
    #[derive(Debug, Clone, Copy)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct InvitationsConfig {
        pub invitation_ttl: Duration<DateLike>,
    }

    mod databases {
        use crate::app_config::prelude::*;

        #[derive(Debug)]
        #[derive(Validate, serdev::Deserialize)]
        // NOTE: Disabled to allow using `main` as default for `main_read`
        //   and `main_write`. Figment is additive by construction therefore
        //   it’s not trivial to remove a key. This is a quick fix.
        // #[serde(deny_unknown_fields)]
        #[serde(validate = "Validate::validate")]
        pub struct DatabasesConfig {
            #[validate(nested)]
            pub main_read: DatabaseConfig,

            #[validate(nested)]
            pub main_write: DatabaseConfig,
        }

        impl DatabasesConfig {
            pub fn main_url(&self) -> &String {
                assert_eq!(self.main_read.url, self.main_write.url);
                &self.main_read.url
            }
        }

        /// Inspired by <https://github.com/SeaQL/sea-orm/blob/bead32a0d812fd9c80c57e91e956e9d90159e067/sea-orm-rocket/lib/src/config.rs>.
        #[derive(Debug)]
        #[serde_with::skip_serializing_none]
        #[derive(Validate, serdev::Deserialize, Serialize)]
        #[serde(deny_unknown_fields)]
        #[serde(validate = "Validate::validate")]
        pub struct DatabaseConfig {
            #[validate(length(min = 1, max = 1024))]
            pub url: String,

            #[serde(default)]
            pub min_connections: Option<u32>,

            pub max_connections: usize,

            pub connect_timeout: u64,

            #[serde(default)]
            pub acquire_timeout: Option<u64>,

            #[serde(default)]
            pub idle_timeout: Option<u64>,

            pub sqlx_logging: bool,
        }
    }
}

mod dashboard {
    use std::ops::Deref as _;

    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, Serialize, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct DashboardConfig {
        pub url: DashboardUrl,
    }

    impl AppConfig {
        pub fn dashboard_url(&self) -> &Url {
            self.dashboard.url.deref()
        }
    }
}

mod service_accounts {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ServiceAccountsConfig {
        #[validate(nested)]
        pub prose_workspace: ServiceAccountConfig,
    }

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ServiceAccountConfig {
        pub xmpp_node: JidNode,
    }

    impl AppConfig {
        pub fn workspace_jid(&self) -> BareJid {
            BareJid::from_parts(
                Some(&self.service_accounts.prose_workspace.xmpp_node),
                &self.server.domain,
            )
        }
    }
}

mod bootstrap {
    use super::prelude::*;

    #[derive(Debug, Default)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct BootstrapConfig {}
}

mod pod {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use hickory_proto::rr::Name as DomainName;

    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, Serialize, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    #[cfg_attr(feature = "test", derive(Clone))]
    pub struct PodConfig {
        #[validate(nested)]
        pub address: PodAddress,
    }

    #[derive(Debug, Clone, Default)]
    #[derive(Serialize, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct PodAddress {
        pub domain: Option<DomainName>,

        pub ipv4: Option<Ipv4Addr>,

        pub ipv6: Option<Ipv6Addr>,

        /// NOTE: Here to prevent the creation of an invalid value.
        #[serde(skip)]
        _private: (),
    }

    impl Validate for PodAddress {
        fn validate(&self) -> Result<(), ValidationErrors> {
            if (&self.domain, &self.ipv4, &self.ipv6) == (&None, &None, &None) {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "domain",
                    ValidationError::new("invalid_pod_address").with_message(Cow::Borrowed(
                        "The Pod address must contain at least an IPv4, an IPv6 or a domain.",
                    )),
                );
                Err(errors)
            } else {
                Ok(())
            }
        }
    }

    impl AppConfig {
        /// E.g. `prose.your-company.com.` or `your-company.prose.net.`!
        pub fn pod_fqdn(&self) -> DomainName {
            self.pod.network_address().as_fqdn(&self.server_fqdn())
        }
    }
}

mod server {
    use std::str::FromStr as _;

    use hickory_proto::rr::{Name as DomainName, Name as HostName};

    use crate::server_config::TlsProfile;

    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ServerConfig {
        pub domain: JidDomain,

        #[validate(length(min = 1, max = 1024), non_control_character)]
        pub local_hostname: String,

        pub http_port: u16,

        pub log_level: prosody_config::LogLevel,

        #[validate(nested)]
        pub defaults: ServerDefaultsConfig,
    }

    impl ServerConfig {
        pub fn http_url(&self) -> String {
            format!("http://{}:{}", self.local_hostname, self.http_port)
        }
    }

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ServerDefaultsConfig {
        pub message_archive_enabled: bool,

        pub message_archive_retention: PossiblyInfinite<Duration<DateLike>>,

        pub file_upload_allowed: bool,

        // FIXME: Type strongly when implementing https://github.com/prose-im/prose-pod-api/issues/107.
        pub file_storage_encryption_scheme: String,

        pub file_storage_retention: PossiblyInfinite<Duration<DateLike>>,

        pub mfa_required: bool,

        /// Minimum [TLS](https://fr.wikipedia.org/wiki/Transport_Layer_Security) profile
        /// (see <https://wiki.mozilla.org/Security/Server_Side_TLS>).
        pub tls_profile: TlsProfile,

        pub federation_enabled: bool,

        pub federation_whitelist_enabled: bool,

        pub federation_friendly_servers: LinkedHashSet<JidDomain>,

        // FIXME: Type strongly when implementing https://github.com/prose-im/prose-pod-api/issues/131.
        pub settings_backup_interval: String,

        // FIXME: Type strongly when implementing https://github.com/prose-im/prose-pod-api/issues/131.
        pub user_data_backup_interval: String,

        pub push_notification_with_body: bool,

        pub push_notification_with_sender: bool,
    }

    impl AppConfig {
        pub fn server_domain(&self) -> &JidDomain {
            &self.server.domain
        }

        pub fn groups_domain(&self) -> JidDomain {
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
    }
}

mod server_api {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(validate = "Validate::validate")]
    pub struct ServerApiConfig {
        #[validate(skip)]
        pub port: u16,
    }

    impl AppConfig {
        pub fn server_api_url(&self) -> String {
            format!(
                "http://{}:{}",
                self.server.local_hostname, self.server_api.port
            )
        }
    }
}

mod auth {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct AuthConfig {
        pub token_ttl: iso8601_duration::Duration,

        pub password_reset_token_ttl: iso8601_duration::Duration,

        pub invitation_ttl: iso8601_duration::Duration,
    }
}

mod public_contacts {
    use super::prelude::*;

    #[derive(Debug, Default)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct PublicContactsConfig {
        #[serde(default)]
        pub default: LinkedHashSet<Url>,

        #[serde(default)]
        pub abuse: LinkedHashSet<Url>,

        #[serde(default)]
        pub admin: LinkedHashSet<Url>,

        #[serde(default)]
        pub feedback: LinkedHashSet<Url>,

        #[serde(default)]
        pub sales: LinkedHashSet<Url>,

        #[serde(default)]
        pub security: LinkedHashSet<Url>,

        #[serde(default)]
        pub support: LinkedHashSet<Url>,
    }
}

mod prosody_ext {
    use super::prelude::*;

    /// NOTE: We cannot include [`ProsodySettings`] as a flattened field because
    ///   `#[serde(deny_unknown_fields)]` doesn’t work with `#[serde(flatten)]`.
    ///   See <https://serde.rs/container-attrs.html#deny_unknown_fields>.
    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ProsodyExtConfig {
        pub config_file_path: PathBuf,

        /// NOTE: Those modules will be added to `modules_enabled` after everything
        ///   else has been applied (apart from dynamic overrides, which are always
        ///   applied last).
        #[serde(default)]
        // NOTE: `ValidateRegex` is not implemented for `Vec`,
        //   let’s ignore validating the character set.
        #[validate(custom(function = validate_module_names_vec))]
        pub additional_modules_enabled: Vec<String>,
    }
}

mod prosody {
    pub use prosody_config::ProsodySettings as ProsodyConfig;

    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    // TODO: Add validation to `ProsodyConfig`?
    pub struct ProsodyHostConfig {
        #[serde(default)]
        pub defaults: Option<ProsodyConfig>,

        #[serde(default)]
        pub overrides: Option<ProsodyConfig>,
    }
}

mod branding {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct BrandingConfig {
        #[serde(default)]
        #[validate(length(min = 1, max = 48), non_control_character)]
        pub company_name: Option<String>,

        #[validate(length(min = 1, max = 48), non_control_character)]
        pub api_app_name: String,
    }
}

mod notifiers {
    use crate::invitations::InvitationChannel;

    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct NotifiersConfig {
        pub workspace_invitation_channel: InvitationChannel,

        #[serde(default)]
        #[validate(nested)]
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

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct EmailNotifierConfig {
        pub pod_address: EmailAddress,

        #[validate(length(min = 1, max = 1024))]
        pub smtp_host: String,

        pub smtp_port: u16,

        #[validate(length(min = 1, max = 1024))]
        pub smtp_username: Option<String>,

        #[serde(default)]
        // NOTE: Not validated because of the Rust type system
        //   but it’s a password so let’s ignore it.
        pub smtp_password: Option<SecretString>,

        pub smtp_encrypt: bool,
    }
}

mod debug {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct DebugConfig {
        pub log_config_at_startup: bool,

        pub detailed_error_responses: bool,

        pub c2s_unencrypted: bool,

        // NOTE: Needs to be available in release builds so we can
        //   run the CI in `prose-pod-system` without having to start
        //   live notifiers. Also this turned out very useful
        //   to multiple people so let’s keep it public.
        #[serde(default)]
        #[validate(custom(function = validate_module_names_set))]
        pub skip_startup_actions: HashSet<String>,
    }
}

#[cfg(debug_assertions)]
mod debug_only {
    use super::prelude::*;

    #[derive(Debug, Default)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct DebugOnlyConfig {
        /// When automatically accepting invitations during testing, one might want to authenticate
        /// the created member. With this flag turned on, the member's password will be their JID.
        #[serde(default)]
        pub insecure_password_on_auto_accept_invitation: bool,

        #[serde(default)]
        pub dependency_modes: DependencyModesConfig,
    }

    #[derive(Debug, Clone, Copy)]
    #[derive(serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "snake_case")]
    pub enum NotifierDependencyMode {
        Live,
        Logging,
    }

    impl Default for NotifierDependencyMode {
        fn default() -> Self {
            Self::Live
        }
    }

    #[derive(Debug, Default)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct DependencyModesConfig {
        #[serde(default)]
        pub notifier: NotifierDependencyMode,
    }
}

// MARK: - Helpers

mod util {
    use super::prelude::*;

    pub fn validate_module_names_set(vec: &HashSet<String>) -> Result<(), ValidationError> {
        validate_module_names_iter(vec.iter())
    }
    pub fn validate_module_names_vec(vec: &Vec<String>) -> Result<(), ValidationError> {
        validate_module_names_iter(vec.iter())
    }
    pub fn validate_module_names_iter<'a>(
        iter: impl Iterator<Item = &'a String>,
    ) -> Result<(), ValidationError> {
        for module_name in iter {
            if let Err(e) = validate_module_name(module_name) {
                return Err(e);
            }
        }

        Ok(())
    }

    pub fn validate_module_name(str: &str) -> Result<(), ValidationError> {
        fn is_valid_module_name_char(char: char) -> bool {
            char.is_ascii_lowercase() || char.is_ascii_digit() || char == '_'
        }

        if str.chars().all(is_valid_module_name_char) {
            Ok(())
        } else {
            Err(ValidationError::new("invalid_module_name")
                .with_message(Cow::Owned(format!("'{str}' is not a valid module name"))))
        }
    }
}

// MARK: - Tests

#[cfg(test)]
mod tests {
    use figment::providers::{Format, Json};
    use serde_json::json;

    use super::default_config_static;
    use super::AppConfig;
    use super::DEFAULT_DB_MAX_READ_CONNECTIONS;

    #[inline]
    fn config_from_json(json: &serde_json::Value) -> Result<AppConfig, String> {
        let json = serde_json::to_string(&json).unwrap();

        let figment = default_config_static().merge(Json::string(&json));

        match AppConfig::from_figment(figment) {
            Ok(app_config) => Ok(app_config),
            Err(err) => Err(format!("{err:#}")),
        }
    }

    #[inline]
    fn minimal_config_json() -> serde_json::Value {
        json!({
            "server": {
                "domain": "example.org"
            }
        })
    }

    #[test]
    fn test_minimal_config_empty() {
        let empty_config = json!({});
        assert_ne!(config_from_json(&empty_config).err(), None);
    }

    #[test]
    fn test_minimal_config_ok() {
        let minimal_config = minimal_config_json();
        assert_eq!(config_from_json(&minimal_config).err(), None);
    }

    #[test]
    fn test_default_pod_email_address_no_email_notifier() {
        let minimal_config = minimal_config_json();
        let app_config = config_from_json(&minimal_config).unwrap();
        assert_eq!(
            (app_config.notifiers.email)
                .as_ref()
                .map(|cfg| format!("{cfg:#?}")),
            None
        );
    }

    #[test]
    fn test_default_pod_email_address_ok() {
        let config_json = json!({
            "server": { "domain": "example.org" },
            "notifiers": {
                "email": {
                    "smtp_host": "mail.example.org"
                }
            }
        });
        let app_config = config_from_json(&config_json).unwrap();
        let email_config = app_config.notifiers.email().unwrap();
        assert_eq!(&email_config.pod_address.to_string(), "prose@example.org");
    }

    #[inline]
    fn test_default_pod_domain_(config_json: serde_json::Value, expected: Option<&str>) {
        let app_config = config_from_json(&config_json).unwrap();
        let pod_domain = app_config.pod.address.domain.as_ref();
        assert_eq!(
            pod_domain.map(ToString::to_string),
            expected.map(ToOwned::to_owned)
        );
    }

    #[test]
    fn test_default_pod_domain_ok() {
        test_default_pod_domain_(
            json!({
                "server": { "domain": "example.org" }
            }),
            Some("prose.example.org"),
        )
    }

    #[test]
    fn test_default_pod_domain_ipv4() {
        test_default_pod_domain_(
            json!({
                "server": { "domain": "example.org" },
                "pod": {
                    "address": { "ipv4": "127.0.0.1" }
                },
                "dashboard": {
                    "url": "https://admin.prose.example.org"
                }
            }),
            None,
        )
    }

    #[test]
    fn test_default_pod_domain_ipv6() {
        test_default_pod_domain_(
            json!({
                "server": { "domain": "example.org" },
                "pod": {
                    "address": { "ipv6": "::1" }
                },
                "dashboard": {
                    "url": "https://admin.prose.example.org"
                }
            }),
            None,
        )
    }

    #[test]
    fn test_default_pod_domain_override() {
        test_default_pod_domain_(
            json!({
                "server": { "domain": "example.org" },
                "pod": {
                    "address": { "domain": "chat.example.org" }
                }
            }),
            Some("chat.example.org"),
        );
    }

    #[inline]
    fn test_default_dashboard_url_(config_json: serde_json::Value, expected: &str) {
        let app_config = config_from_json(&config_json).unwrap();
        assert_eq!(app_config.dashboard.url.as_str(), expected);
    }

    #[test]
    fn test_default_dashboard_url_ok() {
        test_default_dashboard_url_(
            json!({
                "server": { "domain": "example.org" }
            }),
            "https://admin.prose.example.org/",
        )
    }

    #[test]
    fn test_default_dashboard_url_domain_overriden() {
        test_default_dashboard_url_(
            json!({
                "server": { "domain": "example.org" },
                "pod": {
                    "address": { "domain": "chat.example.org" }
                }
            }),
            "https://admin.chat.example.org/",
        )
    }

    // NOTE: Also tests the addition of the trailing slash (`/`).
    #[test]
    fn test_default_dashboard_url_override() {
        test_default_dashboard_url_(
            json!({
                "server": { "domain": "example.org" },
                "dashboard": {
                    "url": "https://admin.example.org"
                }
            }),
            "https://admin.example.org/",
        )
    }

    #[test]
    fn test_default_email_notifier_config() {
        let config_json = json!({
            "server": { "domain": "example.org" },
            "notifiers": {
                "email": {
                    "smtp_host": "mail.example.org"
                },
            }
        });
        let app_config = config_from_json(&config_json).unwrap();
        let email_config = app_config.notifiers.email.as_ref().unwrap();
        assert_eq!(email_config.smtp_port, 587);
        assert_eq!(email_config.smtp_encrypt, true);
    }

    #[inline]
    fn test_database_rw_defaults_(input: serde_json::Value, output: serde_json::Value) {
        let config_json = json!({
            "server": { "domain": "example.org" },
            "api": {
                "databases": input,
            }
        });

        let app_config = config_from_json(&config_json);
        assert_eq!(app_config.as_ref().err(), None);

        let json = serde_json::to_string_pretty(&config_json).unwrap();
        let ref db_config = app_config.unwrap().api.databases;

        // NOTE(RemiBardon): Ugly but I don’t care, it just needs to work.
        if let Some(val) = output["main_read"]["url"].as_str() {
            assert_eq!(db_config.main_read.url, val, "main_read.url: {json}");
        }
        if let Some(val) = output["main_read"]["max_connections"].as_u64() {
            assert_eq!(
                db_config.main_read.max_connections as u64, val,
                "main_read.max_connections: {json}"
            );
        }
        if let Some(val) = output["main_write"]["url"].as_str() {
            assert_eq!(db_config.main_write.url, val, "main_write.url: {json}");
        }
        if let Some(val) = output["main_write"]["max_connections"].as_u64() {
            assert_eq!(
                db_config.main_write.max_connections as u64, val,
                "main_write.max_connections: {json}"
            );
        }
    }

    #[test]
    fn test_database_rw_defaults_empty() {
        test_database_rw_defaults_(
            json!({}),
            json!({
                "main_read": {
                    "max_connections": *DEFAULT_DB_MAX_READ_CONNECTIONS
                },
                "main_write": {
                    "max_connections": 1
                },
            }),
        );
    }

    #[test]
    fn test_database_rw_defaults_url_override() {
        test_database_rw_defaults_(
            json!({
                "main": {
                    "url": "example"
                }
            }),
            json!({
                "main_read": {
                    "url": "example",
                    "max_connections": *DEFAULT_DB_MAX_READ_CONNECTIONS
                },
                "main_write": {
                    "url": "example",
                    "max_connections": 1
                },
            }),
        );
    }

    #[test]
    fn test_database_rw_defaults_max_connections_override() {
        test_database_rw_defaults_(
            json!({
                "main": {
                    "url": "example",
                    "max_connections": 4
                }
            }),
            json!({
                "main_read": {
                    "url": "example",
                    "max_connections": 4
                },
                "main_write": {
                    "url": "example",
                    "max_connections": 4
                },
            }),
        );
    }

    #[test]
    fn test_database_rw_defaults_max_connections_override_write() {
        test_database_rw_defaults_(
            json!({
                "main": {
                    "url": "example",
                    "max_connections": 4
                },
                "main_read": {
                    "url": "other"
                },
                "main_write": {
                    "max_connections": 1
                }
            }),
            json!({
                "main_read": {
                    "url": "other",
                    "max_connections": 4
                },
                "main_write": {
                    "url": "example",
                    "max_connections": 1
                },
            }),
        );
    }
}
