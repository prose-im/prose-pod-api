// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::PathBuf;

use ::entity::model::server_config::*;
use ::entity::model::*;

use super::prosody_config_file::model::utils::*;
use super::prosody_config_file::model::ProsodyConfigSection as ProsodyConfigFileSection;
use super::prosody_config_file::model::*;

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
    pub global_settings: ProsodySettings,
    pub additional_sections: Vec<ProsodyConfigSection>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProsodyConfigSection {
    VirtualHost {
        hostname: String,
        settings: ProsodySettings,
    },
    Component {
        hostname: String,
        plugin: String,
        name: String,
        settings: ProsodySettings,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProsodySettings {
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
    pub archive_expires_after: PossiblyInfinite<Duration<DateLike>>,
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
    FilePath(PathBuf),
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
            global_settings: ProsodySettings {
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
                archive_expires_after: PossiblyInfinite::Infinite,
                default_archive_policy: true,
                max_archive_query_results: 100,
                upgrade_legacy_vcards: true,
                groups_file: "/etc/prosody/roster_groups.txt".into(),
            },
            additional_sections: vec![],
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::Raw(LogLevelValue::FilePath("prosody.log".into()))
    }
}

// ===== Config to config file =====

impl Into<Vec<Group<LuaDefinition>>> for ProsodySettings {
    fn into(self) -> Vec<Group<LuaDefinition>> {
        let mut res: Vec<Group<LuaDefinition>> = vec![];

        res.push(def("pidfile", self.pidfile).into());
        res.push(
            vec![
                def("authentication", self.authentication),
                def("storage", self.storage),
            ]
            .into(),
        );
        res.push(def("log", self.log).into());
        res.push(Group::new(
            "Network interfaces/ports",
            vec![
                def("interfaces", self.interfaces),
                def("c2s_ports", self.c2s_ports),
                def("s2s_ports", self.s2s_ports),
                def("http_ports", self.http_ports),
                def("http_interfaces", self.http_interfaces),
            ],
        ));
        res.push(
            vec![
                def("https_ports", self.https_ports),
                def("https_interfaces", self.https_interfaces),
            ]
            .into(),
        );
        res.push(Group::new(
            "Modules",
            vec![
                def("modules_enabled", self.modules_enabled),
                def("modules_disabled", self.modules_disabled),
            ],
        ));
        res.push(
            def("ssl", self.ssl)
                .comment("Path to SSL key and certificate for all server domains")
                .into(),
        );
        res.push(
            def("allow_registration", false)
                .comment("Disable in-band registrations (done through the Prose Pod Dashboard/API)")
                .into(),
        );
        res.push(Group::new(
            "Mandate highest security levels",
            vec![
                def("c2s_require_encryption", self.c2s_require_encryption),
                def("s2s_require_encryption", self.s2s_require_encryption),
                def("s2s_secure_auth", self.s2s_secure_auth),
            ],
        ));
        res.push(Group::new(
            "Enforce safety C2S/S2S limits",
            vec![
                def("c2s_stanza_size_limit", self.c2s_stanza_size_limit),
                def("s2s_stanza_size_limit", self.s2s_stanza_size_limit),
            ],
        ));
        res.push(def("limits", self.limits).into());
        res.push(Group::new(
            "Allow reverse-proxying to WebSocket service over insecure local HTTP",
            vec![
                def("consider_websocket_secure", self.consider_websocket_secure),
                def("cross_domain_websocket", self.cross_domain_websocket),
            ],
        ));
        res.push(
            def("contact_info", self.contact_info)
                .comment("Specify server administrator")
                .into(),
        );
        res.push(Group::new(
            "MAM settings",
            vec![
                def("archive_expires_after", self.archive_expires_after),
                def("default_archive_policy", self.default_archive_policy),
                def("max_archive_query_results", self.max_archive_query_results),
            ],
        ));
        res.push(
            def("upgrade_legacy_vcards", self.upgrade_legacy_vcards)
                .comment("Enable vCard legacy compatibility layer")
                .into(),
        );
        res.push(
            def("groups_file", self.groups_file)
                .comment("Define server members groups file")
                .into(),
        );

        res
    }
}

impl Into<ProsodyConfigFileSection> for ProsodyConfigSection {
    fn into(self) -> ProsodyConfigFileSection {
        match self {
            Self::VirtualHost { hostname, settings } => ProsodyConfigFileSection::VirtualHost {
                comments: vec![],
                hostname,
                settings: settings.into(),
            },
            Self::Component {
                hostname,
                plugin,
                name,
                settings,
            } => ProsodyConfigFileSection::Component {
                comments: vec![],
                hostname,
                plugin,
                name,
                settings: settings.into(),
            },
        }
    }
}

impl Into<ProsodyConfigFile> for ProsodyConfig {
    fn into(self) -> ProsodyConfigFile {
        ProsodyConfigFile {
            header: Some(vec![
                "Prose Pod Server".into(),
                "XMPP Server Configuration".into(),
                r#"/!\ This file has been automatically generated by Prose Pod API."#.into(),
                r#"/!\ Do NOT edit this file manually or your changes will be overriden during the next reload."#.into(),
            ].into()),
            global_settings: self.global_settings.into(),
            additional_sections: self
                .additional_sections
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl Into<LuaValue> for AuthenticationProvider {
    fn into(self) -> LuaValue {
        match self {
            Self::InternalPlain => "internal_plain",
            Self::InternalHashed => "internal_hashed",
            Self::Cyrus => "cyrus",
            Self::LDAP => "ldap",
            Self::Anonymous => "anonymous",
        }
        .into()
    }
}

impl Into<LuaValue> for StorageConfig {
    fn into(self) -> LuaValue {
        match self {
            Self::Raw(backend) => backend.into(),
            Self::Map(map) => map.into(),
        }
    }
}

impl Into<LuaValue> for StorageBackend {
    fn into(self) -> LuaValue {
        match self {
            Self::Internal => "internal",
            Self::SQL => "sql",
            Self::Memory => "memory",
            Self::Null => "null",
            Self::None => "none",
        }
        .into()
    }
}

impl Into<LuaValue> for LogConfig {
    fn into(self) -> LuaValue {
        match self {
            Self::Raw(value) => value.into(),
            Self::Map(map) => map.into(),
        }
    }
}

impl Into<LuaValue> for LogLevelValue {
    fn into(self) -> LuaValue {
        match self {
            Self::FilePath(path) => path.into(),
            Self::Console => "*console".into(),
            Self::Syslog => "*syslog".into(),
        }
    }
}

impl Into<String> for LogLevel {
    fn into(self) -> String {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
        .to_string()
    }
}

impl Into<LuaValue> for Interface {
    fn into(self) -> LuaValue {
        match self {
            Self::AllIPv4 => "*".into(),
            Self::AllIPv6 => "::".into(),
            Self::Address(address) => address.into(),
        }
    }
}

impl Into<LuaValue> for SSLConfig {
    fn into(self) -> LuaValue {
        match self {
            SSLConfig::Automatic(path) => path.into(),
            SSLConfig::Manual { certificate, key } => vec![
                ("certificate", certificate),
                ("key", key),
            ]
            .into(),
        }
    }
}

impl Into<LuaNumber> for Bytes {
    fn into(self) -> LuaNumber {
        match self {
            Self::Bytes(n) => LuaNumber::from(n),
            Self::KiloBytes(n) => mult(Self::Bytes(n), 1000),
            Self::KibiBytes(n) => mult(Self::Bytes(n), 1024),
            Self::MegaBytes(n) => mult(Self::KiloBytes(n), 1000),
            Self::MebiBytes(n) => mult(Self::KibiBytes(n), 1024),
        }
    }
}

impl Into<LuaValue> for DataRate {
    /// <https://prosody.im/doc/modules/mod_limits> states:
    ///
    /// > All units are in terms of bytes, not bits, so that “kb/s” is interpreted as “kilobytes per second”, where a kilobyte is 1000 bytes.
    ///
    /// This behavior is non-standard (we would expect "kB/s" for “kilobytes per second”),
    /// therefore we have to define a custom printer.
    fn into(self) -> LuaValue {
        match self {
            Self::BytesPerSec(n) => format!("{n}b/s"),
            Self::KiloBytesPerSec(n) => format!("{n}kb/s"),
            Self::MegaBytesPerSec(n) => format!("{n}mb/s"),
        }
        .into()
    }
}

impl<Content: DurationContent> Into<LuaValue> for Duration<Content>
where
    Content: DurationContent + Into<LuaValue>,
{
    fn into(self) -> LuaValue {
        self.0.into()
    }
}

impl Into<LuaValue> for DateLike {
    /// Format defined in <https://prosody.im/doc/modules/mod_mam#archive_expiry>.
    fn into(self) -> LuaValue {
        match self {
            Self::Days(n) => format!("{n}d"),
            Self::Weeks(n) => format!("{n}w"),
            Self::Months(n) => format!("{n}m"),
            Self::Years(n) => format!("{n}y"),
        }
        .into()
    }
}

impl<Content> Into<LuaValue> for PossiblyInfinite<Duration<Content>>
where
    Content: DurationContent + Into<LuaValue>,
{
    /// Format defined in <https://prosody.im/doc/modules/mod_mam#archive_expiry>.
    fn into(self) -> LuaValue {
        match self {
            PossiblyInfinite::Infinite => "never".into(),
            PossiblyInfinite::Finite(duration) => duration.into(),
        }
    }
}

impl Into<String> for ConnectionType {
    fn into(self) -> String {
        match self {
            Self::ClientToServer => "c2s",
            Self::ServerToServerInbounds => "s2sin",
            Self::ServerToServerOutbounds => "s2sout",
        }
        .to_string()
    }
}

impl Into<LuaValue> for ConnectionLimits {
    fn into(self) -> LuaValue {
        let mut map: LinkedHashMap<String, LuaValue> = LinkedHashMap::new();
        if let Some(rate) = self.rate {
            map.insert("rate".to_string(), rate.into());
        }
        if let Some(burst) = self.burst {
            map.insert("burst".to_string(), format!("{}s", burst.seconds()).into());
        }
        map.into()
    }
}

impl Into<LuaValue> for ContactInfo {
    fn into(self) -> LuaValue {
        vec![
            ("abuse", self.abuse),
            ("admin", self.admin),
            ("feedback", self.feedback),
            ("sales", self.sales),
            ("security", self.security),
            ("support", self.support),
        ]
        .into()
    }
}
