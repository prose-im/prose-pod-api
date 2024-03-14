// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod conversion;

use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use std::hash::Hash;
use std::path::PathBuf;

use crate::model::*;

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

#[derive(Debug, Eq, PartialEq, Default)]
pub struct ProsodySettings {
    pub pidfile: Option<PathBuf>,
    pub authentication: Option<AuthenticationProvider>,
    pub storage: Option<StorageConfig>,
    pub log: Option<LogConfig>,
    pub interfaces: Option<Vec<Interface>>,
    pub c2s_ports: Option<Vec<u16>>,
    pub s2s_ports: Option<Vec<u16>>,
    pub http_ports: Option<Vec<u16>>,
    pub http_interfaces: Option<Vec<Interface>>,
    pub https_ports: Option<Vec<u16>>,
    pub https_interfaces: Option<Vec<Interface>>,
    pub admins: Option<LinkedHashSet<JID>>,
    pub modules_enabled: Option<LinkedHashSet<String>>,
    pub modules_disabled: Option<LinkedHashSet<String>>,
    pub ssl: Option<SSLConfig>,
    /// See <https://prosody.im/doc/creating_accounts#in-band_registration>.
    pub allow_registration: Option<bool>,
    pub c2s_require_encryption: Option<bool>,
    pub s2s_require_encryption: Option<bool>,
    pub s2s_secure_auth: Option<bool>,
    pub c2s_stanza_size_limit: Option<Bytes>,
    pub s2s_stanza_size_limit: Option<Bytes>,
    pub limits: Option<LinkedHashMap<ConnectionType, ConnectionLimits>>,
    pub consider_websocket_secure: Option<bool>,
    pub cross_domain_websocket: Option<bool>,
    pub contact_info: Option<ContactInfo>,
    pub archive_expires_after: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub default_archive_policy: Option<bool>,
    pub max_archive_query_results: Option<u32>,
    pub upgrade_legacy_vcards: Option<bool>,
    pub groups_file: Option<PathBuf>,
    pub http_file_share_size_limit: Option<Bytes>,
    pub http_file_share_daily_quota: Option<Bytes>,
    pub http_file_share_expires_after: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub http_host: Option<String>,
    pub http_external_url: Option<String>,
    /// See <https://prosody.im/doc/chatrooms#creating_rooms>.
    pub restrict_room_creation: Option<RoomCreationRestriction>,
    /// See <https://prosody.im/doc/modules/mod_muc_mam>.
    pub log_all_rooms: Option<bool>,
    /// See <https://prosody.im/doc/modules/mod_muc_mam>.
    pub muc_log_expires_after: Option<PossiblyInfinite<Duration<DateLike>>>,
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

/// See <https://prosody.im/doc/chatrooms#creating_rooms>.
#[derive(Debug, Eq, PartialEq)]
pub enum RoomCreationRestriction {
    /// Restrict room creation to server admins defined in the Prosody config.
    AdminsOnly,
    /// Restrict the creation of rooms to users on the main domain only
    /// (e.g. `example.com` in the case `Component "conference.example.com" "muc"`).
    DomainOnly,
}

// ===== DEFAULT =====

impl Default for LogConfig {
    fn default() -> Self {
        Self::Raw(LogLevelValue::FilePath("prosody.log".into()))
    }
}
