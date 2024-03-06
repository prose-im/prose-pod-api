// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::path::PathBuf;

use ::entity::model::server_config::*;
use ::entity::model::*;

/// Prosody configuration.
///
/// This data structure represents a Prosody configuration file in a type-safe way.
/// It is an intermediate representation between the database model and the AST-like
/// `ProsodyConfigFile`. The latter is used to generate the text-based configuration file.
///
/// If we ever want to add "advanced" routes for users to directly edit their server's
/// Prosody configuration, the types defined in this module could be used for safer parsing.
///
/// > NOTE: Only non-optional fields, fields configurable via the Prose Pod API and fields
/// > we would like to override in the configuration file are defined, as the rest will use
/// > Prosody defaults.
///
/// See <https://prosody.im/doc/configure>.
#[derive(Debug, Eq, PartialEq)]
pub struct ProsodyConfig {
    pub pidfile: PathBuf,
    pub authentication: AuthenticationProvider,
    pub storage: StorageConfig,
    pub log: LogConfig,
    pub interfaces: Vec<Interface>,
    pub c2s_ports: Vec<u16>,
    pub s2s_ports: Vec<u16>,
    pub http_ports: Vec<u16>,
    pub http_interfaces: Vec<Interface>,
    pub https_ports: Vec<u16>,
    pub https_interfaces: Vec<Interface>,
    pub admins: HashSet<JID>,
    pub modules_enabled: HashSet<String>,
    pub modules_disabled: HashSet<String>,
    pub ssl: SSLConfig,
    /// See <https://prosody.im/doc/creating_accounts#in-band_registration>.
    pub allow_registration: bool,
    pub c2s_require_encryption: bool,
    pub s2s_require_encryption: bool,
    pub s2s_secure_auth: bool,
    pub c2s_stanza_size_limit: Bytes,
    pub s2s_stanza_size_limit: Bytes,
    pub limits: LinkedHashMap<ConnectionType, ConnectionLimits>,
    pub consider_websocket_secure: bool,
    pub cross_domain_websocket: bool,
    pub contact_info: ContactInfo,
    pub archive_expires_after: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub default_archive_policy: bool,
    pub max_archive_query_results: u32,
    pub upgrade_legacy_vcards: bool,
    pub groups_file: PathBuf,
}

/// See <https://prosody.im/doc/authentication#providers>.
#[derive(Debug, Eq, PartialEq)]
pub enum AuthenticationProvider {
    /// Plaintext passwords stored using built-in storage.
    InternalPlain,
    /// Hashed passwords stored using built-in storage.
    InternalHashed,
    /// Cyrus SASL integration (LDAP, PAM, …).
    Cyrus,
    /// Authenticate users against an LDAP directory using lua-ldap.
    LDAP,
    /// SASL 'ANONYMOUS' mechanism, random username, requires no credentials.
    Anonymous,
}

/// See <https://prosody.im/doc/storage>.
#[derive(Debug, Eq, PartialEq)]
pub enum StorageConfig {
    /// One value (e.g. `"internal"`, `"sql"`…).
    Raw(StorageBackend),
    /// A map of values (e.g. `storage = {
    ///   roster = "sql";
    /// }`).
    Map(LinkedHashMap<String, StorageBackend>),
}

/// See <https://prosody.im/doc/storage#backends>.
#[derive(Debug, Eq, PartialEq)]
pub enum StorageBackend {
    /// Default file-based storage.
    Internal,
    /// SQL database support.
    SQL,
    /// Keeps data in memory only, intended for tests, **not for production**.
    Memory,
    /// Built-in backend that always fails to load/save data.
    Null,
    /// Backend where all stores are always empty and saving data always fails.
    None,
}

/// See <https://prosody.im/doc/ports#default_interfaces>.
#[derive(Debug, Eq, PartialEq)]
pub enum Interface {
    /// All IPv4 interfaces.
    AllIPv4,
    /// All IPv6 interfaces.
    AllIPv6,
    /// IPv4 or IPv6 address.
    Address(String),
}

/// See <https://prosody.im/doc/logging>.
#[derive(Debug, Eq, PartialEq)]
pub enum LogConfig {
    /// One value (file path, `"*syslog"` or `"*console"`).
    Raw(LogLevelValue),
    /// A map of values (e.g. `{
    ///   info = "*console";
    ///   warn = "*console";
    ///   error = "*console";
    /// }`).
    Map(LinkedHashMap<LogLevel, LogLevelValue>),
}

/// See <https://prosody.im/doc/logging>.
#[derive(Debug, Eq, PartialEq)]
pub enum LogLevelValue {
    /// A file path, relative to the config file.
    FilePath(String),
    /// Log to the console, useful for debugging when running in the foreground (`"*console"`).
    Console,
    /// Log to syslog (`"*syslog"`).
    ///
    /// Requires the `mod_posix` module.
    Syslog,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// See <https://prosody.im/doc/certificates#installing_the_certificate>.
#[derive(Debug, Eq, PartialEq)]
pub enum SSLConfig {
    /// Automatic location.
    ///
    /// > NOTE: If defined to `"/path/to/cert.crt"`, expects `/path/to/cert.key` to also exist.
    Automatic(PathBuf),
    /// Manual location (e.g. `{
    ///   certificate = "/etc/prosody/certs/example.com.crt";
    ///   key = "/etc/prosody/certs/example.com.key";
    /// }`).
    ///
    /// See <https://prosody.im/doc/certificates#manual_location>.
    Manual { certificate: PathBuf, key: PathBuf },
}

/// Values from <https://prosody.im/doc/modules/mod_limits>.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ConnectionType {
    /// "c2s"
    ClientToServer,
    /// "s2sin"
    ServerToServerInbounds,
    /// "s2sout"
    ServerToServerOutbounds,
}

/// See <https://prosody.im/doc/modules/mod_limits>.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct ConnectionLimits {
    pub rate: Option<DataRate>,
    pub burst: Option<Duration<TimeLike>>,
}

/// See <https://prosody.im/doc/modules/mod_server_contact_info#configuration>.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct ContactInfo {
    pub abuse: Vec<String>,
    pub admin: Vec<String>,
    pub feedback: Vec<String>,
    pub sales: Vec<String>,
    pub security: Vec<String>,
    pub support: Vec<String>,
}

// ===== DEFAULT =====

impl Default for ProsodyConfig {
    fn default() -> Self {
        Self {
            pidfile: "/var/run/prosody/prosody.pid".into(),
            authentication: AuthenticationProvider::InternalHashed,
            storage: StorageConfig::Raw(StorageBackend::Internal),
            log: LogConfig::Map(
                vec![
                    (LogLevel::Info, LogLevelValue::Console),
                    (LogLevel::Warn, LogLevelValue::Console),
                    (LogLevel::Error, LogLevelValue::Console),
                ]
                .into_iter()
                .collect(),
            ),
            interfaces: vec![Interface::AllIPv4],
            c2s_ports: vec![5222],
            s2s_ports: vec![5269],
            http_ports: vec![5280],
            http_interfaces: vec![Interface::AllIPv4],
            https_ports: vec![],
            https_interfaces: vec![],
            admins: Default::default(),
            modules_enabled: vec![
                "roster",
                "groups",
                "saslauth",
                "tls",
                "dialback",
                "disco",
                "posix",
                "smacks",
                "private",
                "vcard_legacy",
                "vcard4",
                "version",
                "uptime",
                "time",
                "ping",
                "lastactivity",
                "pep",
                "blocklist",
                "limits",
                "carbons",
                "mam",
                "csi",
                "server_contact_info",
                "websocket",
                "s2s_bidi",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
            modules_disabled: Default::default(),
            ssl: SSLConfig::Manual {
                key: "/etc/prosody/certs/prose.org.local.key".into(),
                certificate: "/etc/prosody/certs/prose.org.local.crt".into(),
            },
            allow_registration: false,
            c2s_require_encryption: true,
            s2s_require_encryption: true,
            s2s_secure_auth: false,
            c2s_stanza_size_limit: Bytes::KibiBytes(256),
            s2s_stanza_size_limit: Bytes::KibiBytes(512),
            limits: vec![
                (
                    ConnectionType::ClientToServer,
                    ConnectionLimits {
                        rate: Some(DataRate::KiloBytesPerSec(50)),
                        burst: Some(Duration(TimeLike::Seconds(2))),
                    },
                ),
                (
                    ConnectionType::ServerToServerInbounds,
                    ConnectionLimits {
                        rate: Some(DataRate::KiloBytesPerSec(250)),
                        burst: Some(Duration(TimeLike::Seconds(4))),
                    },
                ),
            ]
            .into_iter()
            .collect(),
            consider_websocket_secure: true,
            cross_domain_websocket: true,
            contact_info: ContactInfo {
                admin: vec!["mailto:hostmaster@prose.org.local".to_string()],
                ..Default::default()
            },
            archive_expires_after: Some(PossiblyInfinite::Infinite),
            default_archive_policy: true,
            max_archive_query_results: 100,
            upgrade_legacy_vcards: true,
            groups_file: "/etc/prosody/roster_groups.txt".into(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::Raw(LogLevelValue::FilePath("prosody.log".to_string()))
    }
}

// ===== INTO =====

impl Into<ConnectionType> for entity::model::server_config::ConnectionType {
    fn into(self) -> ConnectionType {
        match self {
            Self::ClientToServer => ConnectionType::ClientToServer,
            Self::ServerToServerInbounds => ConnectionType::ServerToServerInbounds,
            Self::ServerToServerOutbounds => ConnectionType::ServerToServerOutbounds,
        }
    }
}

mod transform {
    use super::*;
    use std::hash::Hash;

    impl ToString for ProsodyConfig {
        fn to_string(&self) -> String {
            let mut file = format!(
                "-- This file has been automatically generated by Prose Pod API.
    -- Do NOT edit this file manually or your changes will be overriden during the next reload.

    log = {log}

    admins = {{{admins}}}

    modules_enabled = {{{modules_enabled}}}
    modules_disabled = {{{modules_disabled}}}

    limits = {{{limits}}}",
                log = self.log,
                admins = format_set(&self.admins),
                modules_enabled = format_set(&self.modules_enabled),
                modules_disabled = format_set(&self.modules_disabled),
                limits = format_map(&self.limits),
            );

            if let Some(duration) = &self.archive_expires_after {
                file.push_str(&format!(
                    "\n\narchive_expires_after = {}",
                    format_duration_date_inf(duration)
                ));
            }

            file
        }
    }

    fn format_set<T>(set: &HashSet<T>) -> String
    where
        T: Display,
    {
        set.into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_map<K, V>(map: &LinkedHashMap<K, V>) -> String
    where
        K: Display + Hash + Eq,
        V: Display,
    {
        if map.is_empty() {
            return "{}".to_string();
        }

        let inner = map
            .iter()
            .map(|(k, v)| {
                format!("{k} = {v};")
                    .lines()
                    .map(|l| format!("  {l}"))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("{{\n{inner}\n}}")
    }

    /// Format defined in <https://prosody.im/doc/modules/mod_mam#archive_expiry>.
    fn format_duration_date(duration: &Duration<DateLike>) -> String {
        match duration.0 {
            DateLike::Days(n) => format!("{n}d"),
            DateLike::Weeks(n) => format!("{n}w"),
            DateLike::Months(n) => format!("{n}m"),
            DateLike::Years(n) => format!("{n}y"),
        }
    }

    /// Format defined in <https://prosody.im/doc/modules/mod_mam#archive_expiry>.
    fn format_duration_date_inf(duration: &PossiblyInfinite<Duration<DateLike>>) -> String {
        match duration {
            PossiblyInfinite::Infinite => "never".to_string(),
            PossiblyInfinite::Finite(duration) => format_duration_date(duration),
        }
    }

    impl Display for LogConfig {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Raw(v) => v.fmt(f),
                Self::Map(map) => format_map(map).fmt(f),
            }
        }
    }

    impl Display for LogLevelValue {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::FilePath(p) => p.fmt(f),
                Self::Console => write!(f, "*console"),
                Self::Syslog => write!(f, "*syslog"),
            }
        }
    }

    impl Display for LogLevel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Debug => write!(f, "debug"),
                Self::Info => write!(f, "info"),
                Self::Warn => write!(f, "warn"),
                Self::Error => write!(f, "error"),
            }
        }
    }

    impl Display for ConnectionType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::ClientToServer => write!(f, "c2s"),
                Self::ServerToServerInbounds => write!(f, "s2sin"),
                Self::ServerToServerOutbounds => write!(f, "s2sout"),
            }
        }
    }

    /// <https://prosody.im/doc/modules/mod_limits> states:
    ///
    /// > All units are in terms of bytes, not bits, so that “kb/s” is interpreted as “kilobytes per second”, where a kilobyte is 1000 bytes.
    ///
    /// This behavior is non-standard (we would expect "kB/s" for “kilobytes per second”),
    /// therefore we have to define a custom printer.
    fn format_data_rate(rate: &DataRate) -> String {
        match rate {
            DataRate::BytesPerSec(n) => format!("{n}b/s"),
            DataRate::KiloBytesPerSec(n) => format!("{n}kb/s"),
            DataRate::MegaBytesPerSec(n) => format!("{n}mb/s"),
        }
    }

    impl Display for ConnectionLimits {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut map: LinkedHashMap<&str, String> = LinkedHashMap::new();

            if let Some(rate) = &self.rate {
                map.insert("rate", format_data_rate(rate));
            }
            if let Some(burst) = &self.burst {
                map.insert("burst", format!("{}s", burst.seconds()));
            }

            format_map(&map).fmt(f)
        }
    }
}
