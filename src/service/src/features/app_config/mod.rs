// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(crate) mod defaults;

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
pub use self::service_accounts::*;

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

mod prelude {
    pub use std::borrow::Cow;
    pub use std::collections::HashSet;
    pub use std::path::PathBuf;

    pub use linked_hash_set::LinkedHashSet;
    pub use secrecy::SecretString;
    pub use serdev::Serialize;
    pub use validator::{Validate, ValidationError, ValidationErrors};

    pub(crate) use crate::app_config::defaults;
    pub use crate::{errors::MissingConfiguration, models::*};

    pub use super::{util::*, AppConfig};
}

pub mod pub_defaults {
    pub use super::defaults::api::{address as api_address, port as api_port};
}

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
    #[serde(default)]
    pub branding: Arc<BrandingConfig>,

    #[serde(default)]
    pub notifiers: Arc<NotifiersConfig>,

    #[serde(default)]
    pub log: Arc<LogConfig>,

    pub pod: Arc<PodConfig>,

    pub server: Arc<ServerConfig>,

    #[serde(default)]
    pub api: Arc<ApiConfig>,

    pub dashboard: Arc<DashboardConfig>,

    #[serde(default)]
    pub auth: Arc<AuthConfig>,

    #[serde(default)]
    pub public_contacts: Arc<PublicContactsConfig>,

    /// Advanced config, use only if needed.
    #[serde(default)]
    pub prosody_ext: Arc<ProsodyExtConfig>,

    /// Advanced config, use only if needed.
    #[serde(default)]
    pub prosody: HashMap<DomainName, ProsodyHostConfig>,

    /// Advanced config, use only if needed.
    #[serde(default)]
    pub bootstrap: Arc<BootstrapConfig>,

    /// Advanced config, use only if needed.
    #[serde(default)]
    pub service_accounts: Arc<ServiceAccountsConfig>,

    /// Advanced config, use only if needed.
    #[serde(default, rename = "debug_use_at_your_own_risk")]
    pub debug: Arc<DebugConfig>,

    /// Advanced config, use only if needed.
    #[cfg(debug_assertions)]
    #[serde(default)]
    pub debug_only: Arc<DebugOnlyConfig>,
}

impl AppConfig {
    pub fn figment() -> Figment {
        Self::figment_at_path(CONFIG_FILE_PATH.as_path())
    }

    pub fn figment_at_path(path: impl AsRef<Path>) -> Figment {
        use figment::providers::{Env, Format, Toml};

        // NOTE: See what's possible at <https://docs.rs/figment/latest/figment/>.
        Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("PROSE_").split("__"))
    }

    pub fn from_figment(mut figment: Figment) -> anyhow::Result<Self> {
        use anyhow::Context as _;
        use figment::{providers::*, value::Value};

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

        if figment.contains("api.databases.main") {
            // Allow setting configs without expliciting
            // the non-optional `url` field.
            figment = figment.join(Serialized::default(
                "api.databases.main.url",
                defaults::databases::main_url(),
            ));

            // Use `main` as default for `main_read` and `main_write`.
            let main_default = figment.extract_inner::<Value>("api.databases.main").expect(
                "Figment should already contain the key since we just checked with `.contains`.",
            );
            figment = figment
                .join(Serialized::default(
                    "api.databases.main_read",
                    main_default.clone(),
                ))
                .join(Serialized::default(
                    "api.databases.main_write",
                    main_default,
                ));
        }

        // Apply defaults for `main_read` and `main_write`.
        figment = figment
            .join(Serialized::default(
                "api.databases.main_read",
                defaults::databases::main_read(),
            ))
            .join(Serialized::default(
                "api.databases.main_write",
                defaults::databases::main_write(),
            ));

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
}

mod log {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct LogConfig {
        #[serde(default = "defaults::log::level")]
        pub level: LogLevel,

        #[serde(default = "defaults::log::format")]
        pub format: LogFormat,

        #[serde(default = "defaults::log::timer")]
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

        #[serde(default = "crate::app_config::defaults::true_in_debug")]
        pub with_thread_names: bool,
    }

    impl Default for LogConfig {
        fn default() -> Self {
            Self {
                level: defaults::log::level(),
                format: defaults::log::format(),
                timer: defaults::log::timer(),
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
        #[serde(default = "defaults::api::address")]
        #[validate(skip)]
        pub address: IpAddr,

        /// Port to serve on.
        #[serde(default = "defaults::api::port")]
        #[validate(skip)]
        pub port: u16,

        #[validate(nested)]
        pub databases: DatabasesConfig,

        // TODO: Validate.
        #[serde(default)]
        pub network_checks: NetworkChecksConfig,

        // TODO: Validate.
        #[serde(default)]
        pub member_enriching: MemberEnrichingConfig,

        // TODO: Validate.
        #[serde(default)]
        pub invitations: InvitationsConfig,
    }

    impl Default for ApiConfig {
        fn default() -> Self {
            Self {
                address: defaults::api::address(),
                port: defaults::api::port(),
                databases: Default::default(),
                network_checks: Default::default(),
                member_enriching: Default::default(),
                invitations: Default::default(),
            }
        }
    }

    // TODO: Validate values.
    #[derive(Debug, Clone, Copy)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct NetworkChecksConfig {
        #[serde(default = "defaults::api::network_checks::timeout")]
        pub timeout: Duration<TimeLike>,

        #[serde(default = "defaults::api::network_checks::retry_interval")]
        pub retry_interval: Duration<TimeLike>,

        #[serde(default = "defaults::api::network_checks::retry_timeout")]
        pub retry_timeout: Duration<TimeLike>,

        #[serde(default = "defaults::api::network_checks::dns_cache_ttl")]
        pub dns_cache_ttl: Duration<TimeLike>,
    }

    impl Default for NetworkChecksConfig {
        fn default() -> Self {
            use defaults::api::network_checks as defaults;
            Self {
                timeout: defaults::timeout(),
                retry_interval: defaults::retry_interval(),
                retry_timeout: defaults::retry_timeout(),
                dns_cache_ttl: defaults::dns_cache_ttl(),
            }
        }
    }

    // TODO: Validate values.
    #[derive(Debug, Clone, Copy)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MemberEnrichingConfig {
        #[serde(default = "defaults::api::member_enriching::cache_ttl")]
        pub cache_ttl: Duration<TimeLike>,
    }

    impl Default for MemberEnrichingConfig {
        fn default() -> Self {
            use defaults::api::member_enriching as defaults;
            Self {
                cache_ttl: defaults::cache_ttl(),
            }
        }
    }

    // TODO: Validate values.
    #[derive(Debug, Clone, Copy)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct InvitationsConfig {
        #[serde(default = "defaults::api::invitations::invitation_ttl")]
        pub invitation_ttl: Duration<DateLike>,
    }

    impl Default for InvitationsConfig {
        fn default() -> Self {
            use defaults::api::invitations as defaults;
            Self {
                invitation_ttl: defaults::invitation_ttl(),
            }
        }
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

        impl Default for DatabasesConfig {
            fn default() -> Self {
                use defaults::databases as defaults;
                Self {
                    main_read: defaults::main_read(),
                    main_write: defaults::main_write(),
                }
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

            #[serde(default = "defaults::databases::default::max_connections")]
            pub max_connections: usize,

            #[serde(default = "defaults::databases::default::connect_timeout")]
            pub connect_timeout: u64,

            #[serde(default)]
            pub acquire_timeout: Option<u64>,

            #[serde(default)]
            pub idle_timeout: Option<u64>,

            #[serde(default)]
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
    use std::str::FromStr as _;

    use crate::app_config::ADMIN_HOST;
    use crate::models::xmpp::jid::DomainPart;

    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ServiceAccountsConfig {
        #[serde(default = "defaults::service_accounts::prose_pod_api")]
        #[validate(nested)]
        pub prose_pod_api: ServiceAccountConfig,

        #[serde(default = "defaults::service_accounts::prose_workspace")]
        #[validate(nested)]
        pub prose_workspace: ServiceAccountConfig,
    }

    impl Default for ServiceAccountsConfig {
        fn default() -> Self {
            Self {
                prose_pod_api: defaults::service_accounts::prose_pod_api(),
                prose_workspace: defaults::service_accounts::prose_workspace(),
            }
        }
    }

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct ServiceAccountConfig {
        pub xmpp_node: JidNode,
    }

    impl AppConfig {
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
    }
}

mod bootstrap {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct BootstrapConfig {
        #[serde(default = "defaults::bootstrap::prose_pod_api_xmpp_password")]
        pub prose_pod_api_xmpp_password: SecretString,
    }

    impl Default for BootstrapConfig {
        fn default() -> Self {
            use defaults::bootstrap as defaults;
            Self {
                prose_pod_api_xmpp_password: defaults::prose_pod_api_xmpp_password(),
            }
        }
    }
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

        #[serde(default = "defaults::server::local_hostname")]
        #[validate(length(min = 1, max = 1024), non_control_character)]
        pub local_hostname: String,

        #[serde(default = "defaults::server::local_hostname_admin")]
        #[validate(length(min = 1, max = 1024), non_control_character)]
        pub local_hostname_admin: String,

        #[serde(default = "defaults::server::http_port")]
        pub http_port: u16,

        #[serde(default = "defaults::server::log_level")]
        pub log_level: prosody_config::LogLevel,

        #[serde(default)]
        #[validate(nested)]
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

    impl Default for ServerDefaultsConfig {
        fn default() -> Self {
            use defaults::server::defaults;
            Self {
                message_archive_enabled: defaults::message_archive_enabled(),
                message_archive_retention: defaults::message_archive_retention(),
                file_upload_allowed: defaults::file_upload_allowed(),
                file_storage_encryption_scheme: defaults::file_storage_encryption_scheme(),
                file_storage_retention: defaults::file_storage_retention(),
                mfa_required: defaults::mfa_required(),
                tls_profile: defaults::tls_profile(),
                federation_enabled: defaults::federation_enabled(),
                federation_whitelist_enabled: defaults::federation_whitelist_enabled(),
                federation_friendly_servers: defaults::federation_friendly_servers(),
                settings_backup_interval: defaults::settings_backup_interval(),
                user_data_backup_interval: defaults::user_data_backup_interval(),
                push_notification_with_body: defaults::push_notification_with_body(),
                push_notification_with_sender: defaults::push_notification_with_sender(),
            }
        }
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

mod auth {
    use super::prelude::*;

    #[derive(Debug)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct AuthConfig {
        #[serde(default = "defaults::auth::token_ttl")]
        pub token_ttl: iso8601_duration::Duration,

        #[serde(default = "defaults::auth::password_reset_token_ttl")]
        pub password_reset_token_ttl: iso8601_duration::Duration,

        #[serde(default = "defaults::auth::oauth2_registration_key")]
        pub oauth2_registration_key: SecretString,
    }

    impl Default for AuthConfig {
        fn default() -> Self {
            use defaults::auth as defaults;
            Self {
                token_ttl: defaults::token_ttl(),
                password_reset_token_ttl: defaults::password_reset_token_ttl(),
                oauth2_registration_key: defaults::oauth2_registration_key(),
            }
        }
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
        #[serde(default = "defaults::prosody::config_file_path")]
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

    impl Default for ProsodyExtConfig {
        fn default() -> Self {
            use defaults::prosody as defaults;
            Self {
                config_file_path: defaults::config_file_path(),
                additional_modules_enabled: Default::default(),
            }
        }
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

        #[serde(default = "defaults::branding::api_app_name")]
        #[validate(length(min = 1, max = 48), non_control_character)]
        pub api_app_name: String,
    }

    impl Default for BrandingConfig {
        fn default() -> Self {
            use defaults::branding as defaults;
            Self {
                company_name: None,
                api_app_name: defaults::api_app_name(),
            }
        }
    }
}

mod notifiers {
    use crate::invitations::InvitationChannel;

    use super::prelude::*;

    #[derive(Debug, Default)]
    #[derive(Validate, serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(validate = "Validate::validate")]
    pub struct NotifiersConfig {
        #[serde(default = "defaults::notifiers::workspace_invitation_channel")]
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

    impl Default for InvitationChannel {
        fn default() -> Self {
            defaults::notifiers::workspace_invitation_channel()
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

        #[serde(default = "defaults::notifiers::email::smtp_port")]
        pub smtp_port: u16,

        #[validate(length(min = 1, max = 1024))]
        pub smtp_username: Option<String>,
        // NOTE: Not validated because of the Rust type system
        //   but it’s a password so let’s ignore it.
        pub smtp_password: Option<SecretString>,

        #[serde(default = "defaults::notifiers::email::smtp_encrypt")]
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
        #[serde(default = "defaults::true_in_debug")]
        pub log_config_at_startup: bool,

        #[serde(default = "defaults::true_in_debug")]
        pub detailed_error_responses: bool,

        #[serde(default = "defaults::always_false")]
        pub c2s_unencrypted: bool,

        // NOTE: Needs to be available in release builds so we can run the CI in
        //   `prose-pod-system` without having to start live notifiers.
        #[serde(default)]
        #[validate(custom(function = validate_module_names_set))]
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

    #[derive(Debug)]
    #[derive(serdev::Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "snake_case")]
    pub enum UuidDependencyMode {
        Normal,
        Incrementing,
    }

    impl Default for UuidDependencyMode {
        fn default() -> Self {
            Self::Normal
        }
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
        pub uuid: UuidDependencyMode,

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

#[cfg(test)]
mod tests {
    use figment::{
        providers::{Format, Json},
        Figment,
    };
    use serde_json::json;

    use crate::AppConfig;

    #[test]
    fn test_database_rw_defaults() {
        let max_conn_def = crate::app_config::defaults::databases::default::max_connections();

        let cases = vec![
            (
                json!({}),
                json!({
                    "main_read": {
                        "max_connections": max_conn_def
                    },
                    "main_write": {
                        "max_connections": 1
                    },
                }),
            ),
            (
                json!({
                    "main": {
                        "url": "example"
                    }
                }),
                json!({
                    "main_read": {
                        "url": "example",
                        "max_connections": max_conn_def
                    },
                    "main_write": {
                        "url": "example",
                        "max_connections": 1
                    },
                }),
            ),
            (
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
            ),
            (
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
            ),
        ];

        for (input, output) in cases {
            let json = json!({
                "server": {
                    "domain": "example.org"
                },
                "api": {
                    "databases": input,
                }
            });
            let json = serde_json::to_string_pretty(&json).unwrap();

            let figment = Figment::new().merge(Json::string(&json));
            let app_config = AppConfig::from_figment(figment).map_err(|err| err.to_string());
            assert_eq!(app_config.as_ref().err(), None);

            let app_config = app_config.unwrap();
            let ref db_config = app_config.api.databases;

            // NOTE(RemiBardon): Ugly but I don’t care, it just needs to work.
            if let Some(val) = output["main_read"]["url"].as_str() {
                assert_eq!(db_config.main_read.url, val, "main_read.url: {input:#?}");
            }
            if let Some(val) = output["main_read"]["max_connections"].as_u64() {
                assert_eq!(
                    db_config.main_read.max_connections as u64, val,
                    "main_read.max_connections: {input:#?}"
                );
            }
            if let Some(val) = output["main_write"]["url"].as_str() {
                assert_eq!(db_config.main_write.url, val, "main_write.url: {input:#?}");
            }
            if let Some(val) = output["main_write"]["max_connections"].as_u64() {
                assert_eq!(
                    db_config.main_write.max_connections as u64, val,
                    "main_write.max_connections: {input:#?}"
                );
            }
        }
    }
}
