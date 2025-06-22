// prosody-config
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::{Ipv4Addr, Ipv6Addr};

use linked_hash_map::LinkedHashMap;

use super::*;
use crate::model::*;
use crate::prosody_config_file::utils::*;
use crate::prosody_config_file::*;

// ===== Config to config file =====

impl Into<Vec<Group<LuaDefinition>>> for ProsodySettings {
    fn into(self) -> Vec<Group<LuaDefinition>> {
        let mut res: Vec<Group<LuaDefinition>> = vec![];

        let Self {
            pidfile,
            admins,
            authentication,
            default_storage,
            storage,
            storage_archive_item_limit,
            storage_archive_item_limit_cache_size,
            sql,
            sql_manage_tables,
            sqlite_tune,
            log,
            interfaces,
            plugin_paths,
            modules_enabled,
            modules_disabled,
            c2s_direct_tls_ports,
            c2s_ports,
            c2s_timeout,
            c2s_close_timeout,
            c2s_require_encryption,
            c2s_stanza_size_limit,
            c2s_tcp_keepalives,
            s2s_ports,
            s2s_direct_tls_ports,
            s2s_secure_auth,
            s2s_allow_encryption,
            s2s_require_encryption,
            s2s_timeout,
            s2s_close_timeout,
            s2s_secure_domains,
            s2s_insecure_domains,
            s2s_stanza_size_limit,
            s2s_send_queue_size,
            s2s_tcp_keepalives,
            s2s_whitelist,
            ssl,
            ssl_compression,
            ssl_ports,
            allow_registration,
            http_default_host,
            http_max_content_size,
            http_max_buffer_size,
            access_control_allow_methods,
            access_control_allow_headers,
            access_control_allow_origins,
            access_control_allow_credentials,
            access_control_max_age,
            http_default_cors_enabled,
            http_paths,
            http_host,
            http_external_url,
            http_ports,
            http_interfaces,
            https_ports,
            https_interfaces,
            trusted_proxies,
            http_legacy_x_forwarded,
            limits,
            limits_resolution,
            unlimited_jids,
            consider_websocket_secure,
            cross_domain_websocket,
            contact_info,
            archive_expires_after,
            default_archive_policy,
            max_archive_query_results,
            upgrade_legacy_vcards,
            groups_file,
            restrict_room_creation,
            muc_log_all_rooms,
            muc_log_by_default,
            muc_log_expires_after,
            tls_profile,
            reload_modules,
            reload_global_modules,
            http_file_share_secret,
            http_file_share_base_url,
            http_file_share_size_limit,
            http_file_share_allowed_file_types,
            http_file_share_safe_file_types,
            http_file_share_expires_after,
            http_file_share_daily_quota,
            http_file_share_global_quota,
            http_file_share_access,
            use_libevent,
            custom_settings,
            dont_archive_namespaces,
            archive_store,
            mam_include_total,
            archive_cleanup_date_cache_size,
            mam_smart_enable,
        } = self;

        fn push_if_some<T: Into<U>, U>(vec: &mut Vec<U>, value: Option<T>) {
            if let Some(value) = value {
                vec.push(value.into());
            }
        }

        push_if_some(&mut res, option_def(None, "pidfile", pidfile));
        push_if_some(&mut res, option_def(None, "admins", admins));
        push_if_some(&mut res, option_def(None, "authentication", authentication));
        push_if_some(
            &mut res,
            Group::flattened(
                None,
                vec![
                    option_def(None, "default_storage", default_storage),
                    option_def(None, "storage", storage),
                    option_def(
                        None,
                        "storage_archive_item_limit",
                        storage_archive_item_limit,
                    ),
                    option_def(
                        None,
                        "storage_archive_item_limit_cache_size",
                        storage_archive_item_limit_cache_size,
                    ),
                    option_def(None, "sql", sql),
                    option_def(None, "sql_manage_tables", sql_manage_tables),
                    option_def(None, "sqlite_tune", sqlite_tune),
                ],
            ),
        );
        push_if_some(&mut res, option_def(None, "log", log));
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Network interfaces/ports"),
                vec![
                    option_def(None, "interfaces", interfaces),
                    option_def(None, "c2s_ports", c2s_ports),
                    option_def(None, "c2s_direct_tls_ports", c2s_direct_tls_ports),
                    option_def(None, "s2s_ports", s2s_ports),
                    option_def(None, "s2s_direct_tls_ports", s2s_direct_tls_ports),
                    option_def(None, "http_ports", http_ports),
                    option_def(None, "http_interfaces", http_interfaces),
                    option_def(None, "https_ports", https_ports),
                    option_def(None, "https_interfaces", https_interfaces),
                    option_def(None, "ssl_ports", ssl_ports),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Modules"),
                vec![
                    option_def(None, "plugin_paths", plugin_paths),
                    option_def(None, "modules_enabled", modules_enabled),
                    option_def(None, "modules_disabled", modules_disabled),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                None,
                vec![
                    option_def(
                        Some("Path to SSL key and certificate for all server domains"),
                        "ssl",
                        ssl,
                    ),
                    option_def(None, "ssl_compression", ssl_compression),
                ],
            ),
        );
        push_if_some(&mut res, option_def(None, "tls_profile", tls_profile));
        push_if_some(
            &mut res,
            option_def(
                Some("Disable in-band registrations (done through the Prose Pod Dashboard/API)"),
                "allow_registration",
                allow_registration,
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Mandate highest security levels"),
                vec![
                    option_def(None, "c2s_require_encryption", c2s_require_encryption),
                    option_def(None, "s2s_require_encryption", s2s_require_encryption),
                    option_def(None, "s2s_allow_encryption", s2s_allow_encryption),
                    option_def(None, "s2s_secure_auth", s2s_secure_auth),
                    option_def(None, "s2s_secure_domains", s2s_secure_domains),
                    option_def(None, "s2s_insecure_domains", s2s_insecure_domains),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Enforce safety C2S/S2S limits"),
                vec![
                    option_def(None, "c2s_stanza_size_limit", c2s_stanza_size_limit),
                    option_def(None, "s2s_stanza_size_limit", s2s_stanza_size_limit),
                    option_def(None, "s2s_send_queue_size", s2s_send_queue_size),
                    option_def(
                        Some("Avoid federating with the whole XMPP network and only federate with specific servers"),
                        "s2s_whitelist",
                        s2s_whitelist,
                    ),
                ],
            ),
        );

        push_if_some(
            &mut res,
            Group::flattened(
                None,
                vec![
                    option_def(None, "limits", limits),
                    option_def(None, "limits_resolution", limits_resolution),
                    option_def(None, "unlimited_jids", unlimited_jids),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Allow reverse-proxying to WebSocket service over insecure local HTTP"),
                vec![
                    option_def(None, "consider_websocket_secure", consider_websocket_secure),
                    option_def(None, "cross_domain_websocket", cross_domain_websocket),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Timeouts and keepalives"),
                vec![
                    option_def(None, "c2s_timeout", c2s_timeout),
                    option_def(None, "c2s_close_timeout", c2s_close_timeout),
                    option_def(None, "c2s_tcp_keepalives", c2s_tcp_keepalives),
                    option_def(None, "s2s_timeout", s2s_timeout),
                    option_def(None, "s2s_close_timeout", s2s_close_timeout),
                    option_def(None, "s2s_tcp_keepalives", s2s_tcp_keepalives),
                ],
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Specify server administrator"),
                "contact_info",
                contact_info,
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("MAM settings"),
                vec![
                    option_def(None, "archive_expires_after", archive_expires_after),
                    option_def(None, "default_archive_policy", default_archive_policy),
                    option_def(None, "max_archive_query_results", max_archive_query_results),
                    option_def(None, "dont_archive_namespaces", dont_archive_namespaces),
                    option_def(None, "archive_store", archive_store),
                    option_def(None, "mam_include_total", mam_include_total),
                    option_def(
                        None,
                        "archive_cleanup_date_cache_size",
                        archive_cleanup_date_cache_size,
                    ),
                    option_def(None, "mam_smart_enable", mam_smart_enable),
                ],
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Enable vCard legacy compatibility layer"),
                "upgrade_legacy_vcards",
                upgrade_legacy_vcards,
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Define server members groups file"),
                "groups_file",
                groups_file,
            ),
        );

        push_if_some(
            &mut res,
            Group::flattened(
                "HTTP settings".into(),
                vec![
                    option_def(None, "http_host", http_host),
                    option_def(None, "http_external_url", http_external_url),
                    option_def(None, "http_default_host", http_default_host),
                    option_def(None, "http_max_content_size", http_max_content_size),
                    option_def(None, "http_max_buffer_size", http_max_buffer_size),
                    option_def(
                        None,
                        "access_control_allow_methods",
                        access_control_allow_methods,
                    ),
                    option_def(
                        None,
                        "access_control_allow_headers",
                        access_control_allow_headers,
                    ),
                    option_def(
                        None,
                        "access_control_allow_origins",
                        access_control_allow_origins,
                    ),
                    option_def(
                        None,
                        "access_control_allow_credentials",
                        access_control_allow_credentials,
                    ),
                    option_def(None, "access_control_max_age", access_control_max_age),
                    option_def(None, "http_default_cors_enabled", http_default_cors_enabled),
                    option_def(None, "http_paths", http_paths),
                    option_def(None, "trusted_proxies", trusted_proxies),
                    option_def(None, "http_legacy_x_forwarded", http_legacy_x_forwarded),
                ],
            ),
        );
        push_if_some(&mut res, option_def(None, "use_libevent", use_libevent));
        push_if_some(
            &mut res,
            Group::flattened(
                "XEP-0363: HTTP File Upload".into(),
                vec![
                    option_def(
                        Some("Base URL of external upload service"),
                        "http_file_share_base_url",
                        http_file_share_base_url,
                    ),
                    option_def(None, "http_file_share_secret", http_file_share_secret),
                    option_def(
                        None,
                        "http_file_share_size_limit",
                        http_file_share_size_limit,
                    ),
                    option_def(
                        None,
                        "http_file_share_allowed_file_types",
                        http_file_share_allowed_file_types,
                    ),
                    option_def(
                        Some("Safe to show in-line in e.g. browsers"),
                        "http_file_share_safe_file_types",
                        http_file_share_safe_file_types,
                    ),
                    option_def(
                        None,
                        "http_file_share_expires_after",
                        http_file_share_expires_after,
                    ),
                    option_def(
                        None,
                        "http_file_share_daily_quota",
                        http_file_share_daily_quota,
                    ),
                    option_def(
                        None,
                        "http_file_share_global_quota",
                        http_file_share_global_quota,
                    ),
                    option_def(None, "http_file_share_access", http_file_share_access),
                ],
            ),
        );

        push_if_some(
            &mut res,
            option_def(None, "restrict_room_creation", restrict_room_creation),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                "MUC settings".into(),
                vec![
                    option_def(None, "muc_log_all_rooms", muc_log_all_rooms),
                    option_def(None, "muc_log_by_default", muc_log_by_default),
                    option_def(None, "muc_log_expires_after", muc_log_expires_after),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                "mod_reload_modules".into(),
                vec![
                    option_def(None, "reload_modules", reload_modules),
                    option_def(None, "reload_global_modules", reload_global_modules),
                ],
            ),
        );

        res.append(custom_settings.clone().as_mut());

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
            header: None,
            global_settings: self.global_settings.into(),
            additional_sections: self
                .additional_sections
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl ProsodyConfig {
    pub fn print(self, header: Group<LuaComment>) -> ProsodyConfigFile {
        let mut file: ProsodyConfigFile = self.into();
        file.header = Some(header);
        file
    }
}

// ===== LuaValue mappings =====

macro_rules! enum_to_lua_string {
    ($t:ty) => {
        impl Into<LuaValue> for $t {
            fn into(self) -> LuaValue {
                self.to_string().into()
            }
        }
    };
}
macro_rules! insert_in_map {
    ($map:ident $value:ident) => {
        if let Some($value) = $value {
            $map.insert(stringify!($value).to_string(), $value.into());
        }
    };
    ($map:ident $head:ident $(, $tail:ident)* $(,)?) => {
        insert_in_map!($map $head);
        $(insert_in_map!($map $tail);)*
    };
}
macro_rules! struct_to_lua_map {
    ($t:ty, keys: $head:ident $(, $tail:ident)* $(,)?) => {
        impl Into<LuaValue> for $t {
            fn into(self) -> LuaValue {
                let Self {
                    $head,
                    $($tail,)*
                } = self;
                let mut map: LinkedHashMap<String, LuaValue> = LinkedHashMap::new();
                insert_in_map!(map $head, $($tail,)*);
                map.into()
            }
        }
    };
}

impl Into<LuaValue> for BareJid {
    fn into(self) -> LuaValue {
        self.to_string().into()
    }
}

impl Into<LuaValue> for SecretString {
    fn into(self) -> LuaValue {
        use secrecy::ExposeSecret as _;
        LuaValue::String(self.into_secret_string().expose_secret().to_string())
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
enum_to_lua_string!(StorageBackend);

struct_to_lua_map!(SqlConfig, keys:
    driver,
    database,
    host,
    port,
    username,
    password,
);
enum_to_lua_string!(SqlDriver);
enum_to_lua_string!(SqliteTune);

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

impl From<Ipv4Addr> for LuaValue {
    fn from(value: Ipv4Addr) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Ipv6Addr> for LuaValue {
    fn from(value: Ipv6Addr) -> Self {
        Self::String(value.to_string())
    }
}

impl From<IpAddr> for LuaValue {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(ip) => Self::from(ip),
            IpAddr::V6(ip) => Self::from(ip),
        }
    }
}

impl Into<LuaValue> for SSLConfig {
    fn into(self) -> LuaValue {
        let Self {
            certificate,
            key,
            protocol,
            capath,
            cafile,
            verify,
            options,
            depth,
            ciphers,
            dhparam,
            curve,
            verifyext,
            password,
        } = self;
        let mut map: LinkedHashMap<String, LuaValue> = LinkedHashMap::new();
        if let Some(certificate) = certificate {
            map.insert("certificate".to_string(), certificate.into());
        }
        if let Some(key) = key {
            map.insert("key".to_string(), key.into());
        }
        if let Some(protocol) = protocol {
            map.insert("protocol".to_string(), protocol.into());
        }
        if let Some(capath) = capath {
            map.insert("capath".to_string(), capath.into());
        }
        if let Some(cafile) = cafile {
            map.insert("cafile".to_string(), cafile.into());
        }
        if let Some(verify) = verify {
            map.insert("verify".to_string(), verify.into());
        }
        if let Some(options) = options {
            map.insert("options".to_string(), options.into());
        }
        if let Some(depth) = depth {
            map.insert("depth".to_string(), depth.into());
        }
        if let Some(ciphers) = ciphers {
            map.insert("ciphers".to_string(), ciphers.into());
        }
        if let Some(dhparam) = dhparam {
            map.insert("dhparam".to_string(), dhparam.into());
        }
        if let Some(curve) = curve {
            map.insert("curve".to_string(), curve.into());
        }
        if let Some(verifyext) = verifyext {
            map.insert("verifyext".to_string(), verifyext.into());
        }
        if let Some(password) = password {
            map.insert("password".to_string(), password.into());
        }
        map.into()
    }
}

impl Into<LuaValue> for SslProtocol {
    fn into(self) -> LuaValue {
        match self {
            Self::Sslv2 => "sslv2".into(),
            Self::Sslv2OrMore => "sslv2+".into(),
            Self::Sslv3 => "sslv3".into(),
            Self::Sslv3OrMore => "sslv3+".into(),
            Self::Tlsv1 => "tlsv1".into(),
            Self::Tlsv1OrMore => "tlsv1+".into(),
            Self::Tlsv1_1 => "tlsv1_1".into(),
            Self::Tlsv1_1OrMore => "tlsv1_1+".into(),
            Self::Tlsv1_2 => "tlsv1_2".into(),
            Self::Tlsv1_2OrMore => "tlsv1_2+".into(),
            Self::Tlsv1_3 => "tlsv1_3".into(),
            Self::Tlsv1_3OrMore => "tlsv1_3+".into(),
            Self::Other(s) => s.into(),
        }
    }
}

impl Into<LuaValue> for SslVerificationOption {
    fn into(self) -> LuaValue {
        match self {
            Self::None => "none".into(),
            Self::Peer => "peer".into(),
            Self::ClientOnce => "client_once".into(),
            Self::FailIfNoPeerCert => "fail_if_no_peer_cert".into(),
            Self::Other(s) => s.into(),
        }
    }
}

impl Into<LuaValue> for SslOption {
    fn into(self) -> LuaValue {
        self.0.into()
    }
}

impl Into<LuaValue> for ExtraVerificationOption {
    fn into(self) -> LuaValue {
        match self {
            Self::LsecContinue => "lsec_continue".into(),
            Self::LsecIgnorePurpose => "lsec_ignore_purpose".into(),
            Self::Other(s) => s.into(),
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

impl Into<LuaValue> for TimeLike {
    /// Format defined in <https://hg.prosody.im/trunk/file/d7e907830260/spec/util_human_io_spec.lua>.
    fn into(self) -> LuaValue {
        match self {
            Self::Seconds(n) => format!("{n}s"),
            Self::Minutes(n) => format!("{n}min"),
            Self::Hours(n) => format!("{n}h"),
        }
        .into()
    }
}

impl Into<LuaValue> for DateLike {
    /// Format defined in <https://prosody.im/doc/modules/mod_mam#archive_expiry>.
    fn into(self) -> LuaValue {
        match self {
            Self::Days(n) => format!("{n}d"),
            Self::Weeks(n) => format!("{n}w"),
            Self::Months(n) => format!("{n}month"),
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
            Self::Infinite => "never".into(),
            Self::Finite(duration) => duration.into(),
        }
    }
}

impl Into<String> for ConnectionType {
    fn into(self) -> String {
        match self {
            Self::ClientToServer => "c2s",
            Self::ServerToServerInbounds => "s2sin",
            Self::ServerToServerOutbounds => "s2sout",
            Self::Component => "component",
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
        let map: LinkedHashMap<String, LuaValue> = vec![
            ("abuse", self.abuse),
            ("admin", self.admin),
            ("feedback", self.feedback),
            ("sales", self.sales),
            ("security", self.security),
            ("support", self.support),
        ]
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        map.into()
    }
}

impl Into<LuaValue> for RoomCreationRestriction {
    fn into(self) -> LuaValue {
        match self {
            Self::AdminsOnly => true.into(),
            Self::DomainOnly => "local".into(),
        }
    }
}

impl Into<LuaValue> for ArchivePolicy {
    fn into(self) -> LuaValue {
        match self {
            Self::Always => true.into(),
            Self::OnlyIfUserEnabled => false.into(),
            Self::ContactsOnly => "roster".into(),
        }
    }
}

impl Into<LuaValue> for TlsProfile {
    fn into(self) -> LuaValue {
        match self {
            Self::Modern => "modern".into(),
            Self::Intermediate => "intermediate".into(),
            Self::Old => "old".into(),
        }
    }
}

impl Into<LuaValue> for Mime {
    fn into(self) -> LuaValue {
        LuaValue::String(self.to_string())
    }
}

// ===== Data structure conversions =====

impl TimeLike {
    pub fn seconds_as_lua_number(&self) -> LuaNumber {
        match self {
            Self::Seconds(n) => (*n).into(),
            Self::Minutes(n) => mult(*n, 60),
            Self::Hours(n) => mult(*n, Self::Minutes(60).seconds_as_lua_number()),
        }
    }
}

impl DateLike {
    pub fn seconds_as_lua_number(&self) -> LuaNumber {
        mult(
            self.days_as_lua_number(),
            TimeLike::Hours(24).seconds_as_lua_number(),
        )
    }
    pub fn days_as_lua_number(&self) -> LuaNumber {
        match self {
            Self::Days(n) => (*n).into(),
            Self::Weeks(n) => mult(*n, 7),
            Self::Months(n) => mult(*n, 30),
            Self::Years(n) => mult(*n, 365),
        }
    }
}

impl Duration<TimeLike> {
    pub fn seconds_as_lua_number(&self) -> LuaNumber {
        self.0.seconds_as_lua_number()
    }
}

impl PossiblyInfinite<Duration<TimeLike>> {
    pub fn seconds_as_lua_number(self) -> LuaNumber {
        match self {
            Self::Infinite => (-1).into(),
            Self::Finite(duration) => duration.seconds_as_lua_number(),
        }
    }
}

impl Duration<DateLike> {
    pub fn seconds_as_lua_number(&self) -> LuaNumber {
        self.0.seconds_as_lua_number()
    }
    pub fn days_as_lua_number(&self) -> LuaNumber {
        self.0.days_as_lua_number()
    }
}

impl PossiblyInfinite<Duration<DateLike>> {
    pub fn seconds_as_lua_number(self) -> LuaNumber {
        match self {
            Self::Infinite => (-1).into(),
            Self::Finite(duration) => duration.seconds_as_lua_number(),
        }
    }
    pub fn days_as_lua_number(self) -> LuaNumber {
        match self {
            Self::Infinite => (-1).into(),
            Self::Finite(duration) => duration.days_as_lua_number(),
        }
    }
}
