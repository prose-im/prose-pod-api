// prosody-config
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod conversion;

use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use std::hash::Hash;
use std::path::PathBuf;

use crate::prosody_config_file::{Group, LuaDefinition};
use crate::{model::*, LuaValue};

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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProsodyConfig {
    pub global_settings: ProsodySettings,
    pub additional_sections: Vec<ProsodyConfigSection>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl ProsodyConfigSection {
    pub fn hostname(&self) -> &String {
        match self {
            Self::VirtualHost { hostname, .. } => hostname,
            Self::Component { hostname, .. } => hostname,
        }
    }
    pub fn settings(&self) -> &ProsodySettings {
        match self {
            Self::VirtualHost { settings, .. } => settings,
            Self::Component { settings, .. } => settings,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ProsodySettings {
    pub pidfile: Option<PathBuf>,
    pub admins: Option<LinkedHashSet<JID>>,
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
    pub plugin_paths: Option<LinkedHashSet<String>>,
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
    /// See <https://modules.prosody.im/mod_s2s_whitelist>.
    pub s2s_whitelist: Option<LinkedHashSet<String>>,
    pub limits: Option<LinkedHashMap<ConnectionType, ConnectionLimits>>,
    pub consider_websocket_secure: Option<bool>,
    pub cross_domain_websocket: Option<bool>,
    pub contact_info: Option<ContactInfo>,
    pub archive_expires_after: Option<PossiblyInfinite<Duration<DateLike>>>,
    /// Controls whether messages are archived by default.
    ///
    /// See <https://prosody.im/doc/modules/mod_mam>.
    pub default_archive_policy: Option<ArchivePolicy>,
    /// The maxiumum number of messages returned to a client at a time.
    /// Too low will cause excessive queries when clients try to fetch all messages,
    /// too high may consume more resources on the server.
    ///
    /// See <https://prosody.im/doc/modules/mod_mam>.
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
    pub muc_log_all_rooms: Option<bool>,
    /// See <https://prosody.im/doc/modules/mod_muc_mam>.
    pub muc_log_by_default: Option<bool>,
    /// See <https://prosody.im/doc/modules/mod_muc_mam>.
    pub muc_log_expires_after: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub custom_settings: Vec<Group<LuaDefinition>>,
    pub tls_profile: Option<TlsProfile>,
}

impl ProsodySettings {
    pub fn custom_setting(&self, name: &str) -> Option<LuaValue> {
        self.custom_settings
            .iter()
            .flat_map(|c| c.elements.clone())
            .find(|c| c.key == name)
            .map(|d| d.value)
    }

    pub fn enable_module(&mut self, module_name: String) {
        self.unmark_disabled(&module_name);
        self.modules_enabled
            .get_or_insert_default()
            .insert_if_absent(module_name);
    }

    pub fn disable_module(&mut self, module_name: String) {
        self.unmark_enabled(&module_name);
        self.modules_disabled
            .get_or_insert_default()
            .insert_if_absent(module_name);
    }

    pub fn unmark_enabled(&mut self, module_name: &str) {
        if let Some(ref mut modules_enabled) = self.modules_enabled {
            if modules_enabled.remove(module_name) && modules_enabled.is_empty() {
                self.modules_enabled = None;
            }
        }
    }

    pub fn unmark_disabled(&mut self, module_name: &str) {
        if let Some(ref mut modules_disabled) = self.modules_disabled {
            if modules_disabled.remove(module_name) && modules_disabled.is_empty() {
                self.modules_disabled = None;
            }
        }
    }
}

/// See <https://prosody.im/doc/authentication#providers>.
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StorageConfig {
    /// One value (e.g. `"internal"`, `"sql"`…).
    Raw(StorageBackend),
    /// A map of values (e.g. `storage = {
    ///   roster = "sql";
    /// }`).
    Map(LinkedHashMap<String, StorageBackend>),
}

/// See <https://prosody.im/doc/storage#backends>.
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Interface {
    /// All IPv4 interfaces.
    AllIPv4,
    /// All IPv6 interfaces.
    AllIPv6,
    /// IPv4 or IPv6 address.
    Address(String),
}

/// See <https://prosody.im/doc/logging>.
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// See <https://prosody.im/doc/certificates#installing_the_certificate>
/// and <https://prosody.im/doc/advanced_ssl_config#ssl_options>.
///
/// Default source: `core_defaults` in <https://hg.prosody.im/trunk/file/tip/core/certmanager.lua>.
///
/// Example:
///
/// ```lua
/// {
///   certificate = "/etc/prosody/certs/example.com.crt";
///   key = "/etc/prosody/certs/example.com.key";
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct SSLConfig {
    /// Required. Path to your certificate file, relative to your primary config file.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#certificate>.
    pub certificate: Option<PathBuf>,
    /// Required. Path to your private key file, relative to your primary config file.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#key>.
    pub key: Option<PathBuf>,
    /// What handshake to use.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#protocol>.
    pub protocol: Option<SslProtocol>,
    /// Path to directory containing root certificates that you wish Prosody to trust when verifying the certificates of remote servers.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#capath>.
    pub capath: Option<PathBuf>,
    /// Path to a file containing root certificates that you wish Prosody to trust. Similar to `capath` but with all certificates concatenated together.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#cafile>.
    pub cafile: Option<PathBuf>,
    /// See <https://prosody.im/doc/advanced_ssl_config#verify>.
    pub verify: Option<SslVerificationOption>,
    /// See <https://prosody.im/doc/advanced_ssl_config#options>.
    pub options: Option<LinkedHashSet<SslOption>>,
    /// How long a chain of certificate authorities to check when looking for a trusted root certificate.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#depth>.
    pub depth: Option<u8>,
    /// An [OpenSSL cipher string]. This selects what ciphers Prosody will offer to clients, and in what order.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#ciphers>.
    ///
    /// [OpenSSL cipher string]: https://docs.openssl.org/master/man1/openssl-ciphers/#cipher-strings "openssl-ciphers - OpenSSL Documentation"
    pub ciphers: Option<String>,
    /// A path to a file containing parameters for [Diffie–Hellman key exchange].
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#dhparam>.
    ///
    /// [Diffie–Hellman key exchange]: https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange "Diffie–Hellman key exchange | Wikipedia"
    pub dhparam: Option<PathBuf>,
    /// Curve for Elliptic curve Diffie–Hellman.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#curve>.
    pub curve: Option<String>,
    /// A list of “extra” verification options.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#verifyext>.
    pub verifyext: Option<LinkedHashSet<ExtraVerificationOption>>,
    /// Password for encrypted private keys.
    ///
    /// See <https://prosody.im/doc/advanced_ssl_config#password>.
    pub password: Option<SecretString>,
}

/// See <https://prosody.im/doc/advanced_ssl_config#protocol>.
///
/// Source: `protocols` in <https://hg.prosody.im/trunk/file/tip/util/sslconfig.lua>.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SslProtocol {
    /// `"sslv2"`.
    Sslv2,
    /// `"sslv2+"`.
    Sslv2OrMore,
    /// `"sslv3"`.
    Sslv3,
    /// `"sslv3+"`.
    Sslv3OrMore,
    /// `"tlsv1"`.
    Tlsv1,
    /// `"tlsv1+"`.
    Tlsv1OrMore,
    /// `"tlsv1_1"`.
    Tlsv1_1,
    /// `"tlsv1_1+"`.
    Tlsv1_1OrMore,
    /// `"tlsv1_2"`.
    Tlsv1_2,
    /// `"tlsv1_2+"`.
    Tlsv1_2OrMore,
    /// `"tlsv1_3"`.
    Tlsv1_3,
    /// `"tlsv1_3+"`.
    Tlsv1_3OrMore,
    /// A custom value, for future-proofing.
    Other(&'static str),
}

/// See <https://prosody.im/doc/advanced_ssl_config#verify>.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SslVerificationOption {
    /// No verification.
    None,
    /// Verify the peer’s certificate.
    Peer,
    /// Do not request the client’s certificate during renegotiation.
    ClientOnce,
    /// Fail if the peer does not present a certificate.
    FailIfNoPeerCert,
    /// A custom value, for future-proofing.
    Other(&'static str),
}

/// See <https://prosody.im/doc/advanced_ssl_config#options>
/// and <https://docs.openssl.org/master/man3/SSL_CTX_set_options/#notes>.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SslOption(pub &'static str);

/// Source: <https://github.com/lunarmodules/luasec/blob/master/src/options.c>.
#[allow(non_upper_case_globals)]
pub mod ssl_option {
    use super::SslOption;

    pub const SSL_OP_NO_SSLv2: SslOption = SslOption("no_sslv2");
    pub const SSL_OP_NO_SSLv3: SslOption = SslOption("no_sslv3");
    pub const SSL_OP_NO_TLSv1: SslOption = SslOption("no_tlsv1");
    pub const SSL_OP_NO_TLSv1_1: SslOption = SslOption("no_tlsv1_1");
    pub const SSL_OP_NO_TLSv1_2: SslOption = SslOption("no_tlsv1_2");
    pub const SSL_OP_NO_TLSv1_3: SslOption = SslOption("no_tlsv1_3");
}

/// See <https://prosody.im/doc/advanced_ssl_config#verifyext>.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ExtraVerificationOption {
    /// Don’t fail the handshake when an untrusted/invalid certificate is encountered.
    LsecContinue,
    /// Ignore the certificate’s “purpose” flags.
    LsecIgnorePurpose,
    /// A custom value, for future-proofing.
    Other(&'static str),
}

/// Values from <https://prosody.im/doc/modules/mod_limits>.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ConnectionType {
    /// "c2s"
    ClientToServer,
    /// "s2sin"
    ServerToServerInbounds,
    /// "s2sout"
    ServerToServerOutbounds,
}

/// See <https://prosody.im/doc/modules/mod_limits>.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ConnectionLimits {
    pub rate: Option<DataRate>,
    pub burst: Option<Duration<TimeLike>>,
}

/// See <https://prosody.im/doc/modules/mod_server_contact_info#configuration>.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ContactInfo {
    pub abuse: Vec<String>,
    pub admin: Vec<String>,
    pub feedback: Vec<String>,
    pub sales: Vec<String>,
    pub security: Vec<String>,
    pub support: Vec<String>,
}

/// See <https://prosody.im/doc/chatrooms#creating_rooms>.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RoomCreationRestriction {
    /// Restrict room creation to server admins defined in the Prosody config.
    AdminsOnly,
    /// Restrict the creation of rooms to users on the main domain only
    /// (e.g. `example.com` in the case `Component "conference.example.com" "muc"`).
    DomainOnly,
}

/// See <https://prosody.im/doc/modules/mod_mam>.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArchivePolicy {
    /// Always archive messages.
    Always,
    /// Only archive messages if the user enables it.
    OnlyIfEnabled,
    /// Only archive messages for contacts.
    ContactsOnly,
}

/// See <https://prosody.im/doc/configure#other_encryption_options> and <https://wiki.mozilla.org/Security/Server_Side_TLS>.
///
/// Source: `mozilla_ssl_configs` in <https://hg.prosody.im/trunk/file/tip/core/certmanager.lua>.
#[derive(Debug, Clone, Eq, PartialEq)]
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

// ===== DEFAULT =====

impl Default for LogConfig {
    fn default() -> Self {
        Self::Raw(LogLevelValue::FilePath("prosody.log".into()))
    }
}
