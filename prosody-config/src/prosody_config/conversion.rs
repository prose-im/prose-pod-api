// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_map::LinkedHashMap;

use super::*;
use crate::model::*;
use crate::prosody_config_file::utils::*;
use crate::prosody_config_file::*;

// ===== Config to config file =====

impl Into<Vec<Group<LuaDefinition>>> for ProsodySettings {
    fn into(self) -> Vec<Group<LuaDefinition>> {
        let mut res: Vec<Group<LuaDefinition>> = vec![];

        fn push_if_some<T: Into<U>, U>(vec: &mut Vec<U>, value: Option<T>) {
            if let Some(value) = value {
                vec.push(value.into());
            }
        }

        push_if_some(&mut res, option_def(None, "pidfile", self.pidfile));
        push_if_some(&mut res, option_def(None, "admins", self.admins));
        push_if_some(
            &mut res,
            Group::flattened(
                None,
                vec![
                    option_def(None, "authentication", self.authentication),
                    option_def(None, "storage", self.storage),
                ],
            ),
        );
        push_if_some(&mut res, option_def(None, "log", self.log));
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Network interfaces/ports"),
                vec![
                    option_def(None, "interfaces", self.interfaces),
                    option_def(None, "c2s_ports", self.c2s_ports),
                    option_def(None, "s2s_ports", self.s2s_ports),
                    option_def(None, "http_ports", self.http_ports),
                    option_def(None, "http_interfaces", self.http_interfaces),
                    option_def(None, "https_ports", self.https_ports),
                    option_def(None, "https_interfaces", self.https_interfaces),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Modules"),
                vec![
                    option_def(None, "modules_enabled", self.modules_enabled),
                    option_def(None, "modules_disabled", self.modules_disabled),
                ],
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Path to SSL key and certificate for all server domains"),
                "ssl",
                self.ssl,
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Disable in-band registrations (done through the Prose Pod Dashboard/API)"),
                "allow_registration",
                self.allow_registration,
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Mandate highest security levels"),
                vec![
                    option_def(None, "c2s_require_encryption", self.c2s_require_encryption),
                    option_def(None, "s2s_require_encryption", self.s2s_require_encryption),
                    option_def(None, "s2s_secure_auth", self.s2s_secure_auth),
                ],
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Enforce safety C2S/S2S limits"),
                vec![
                    option_def(None, "c2s_stanza_size_limit", self.c2s_stanza_size_limit),
                    option_def(None, "s2s_stanza_size_limit", self.s2s_stanza_size_limit),
                ],
            ),
        );
        push_if_some(&mut res, option_def(None, "limits", self.limits));
        push_if_some(
            &mut res,
            Group::flattened(
                Some("Allow reverse-proxying to WebSocket service over insecure local HTTP"),
                vec![
                    option_def(
                        None,
                        "consider_websocket_secure",
                        self.consider_websocket_secure,
                    ),
                    option_def(None, "cross_domain_websocket", self.cross_domain_websocket),
                ],
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Specify server administrator"),
                "contact_info",
                self.contact_info,
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                Some("MAM settings"),
                vec![
                    option_def(None, "archive_expires_after", self.archive_expires_after),
                    option_def(None, "default_archive_policy", self.default_archive_policy),
                    option_def(
                        None,
                        "max_archive_query_results",
                        self.max_archive_query_results,
                    ),
                ],
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Enable vCard legacy compatibility layer"),
                "upgrade_legacy_vcards",
                self.upgrade_legacy_vcards,
            ),
        );
        push_if_some(
            &mut res,
            option_def(
                Some("Define server members groups file"),
                "groups_file",
                self.groups_file,
            ),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                "HTTP settings".into(),
                vec![
                    option_def(
                        None,
                        "http_file_share_size_limit",
                        self.http_file_share_size_limit,
                    ),
                    option_def(
                        None,
                        "http_file_share_daily_quota",
                        self.http_file_share_daily_quota,
                    ),
                    option_def(
                        None,
                        "http_file_share_expires_after",
                        self.http_file_share_expires_after
                            // `http_file_share_expires_after` is defined as a number of seconds.
                            // See <https://prosody.im/doc/modules/mod_http_file_share#retention>.
                            .map(|d| d.seconds_as_lua_number()),
                    ),
                    option_def(None, "http_host", self.http_host),
                    option_def(None, "http_external_url", self.http_external_url),
                ],
            ),
        );

        push_if_some(
            &mut res,
            option_def(None, "restrict_room_creation", self.restrict_room_creation),
        );
        push_if_some(
            &mut res,
            Group::flattened(
                "MUC settings".into(),
                vec![
                    option_def(None, "muc_log_all_rooms", self.muc_log_all_rooms),
                    option_def(None, "muc_log_by_default", self.muc_log_by_default),
                    option_def(None, "muc_log_expires_after", self.muc_log_expires_after),
                ],
            ),
        );

        push_if_some(
            &mut res,
            Group::flattened(
                "mod_init_admin".into(),
                vec![
                    option_def(None, "init_admin_jid", self.init_admin_jid),
                    option_def(
                        None,
                        "init_admin_password_env_var_name",
                        self.init_admin_password_env_var_name,
                    ),
                ],
            ),
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

impl Into<LuaValue> for JID {
    fn into(self) -> LuaValue {
        self.to_string().into()
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
