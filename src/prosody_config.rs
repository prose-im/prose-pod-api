// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

use crate::server_ctl::{DataRate, DurationDate, DurationTime, PossiblyInfinite};
use ::model::JID;

/// Prosody configuration.
///
/// NOTE: Only non-optional fields and fields configurable via Prose Pod API are defined,
///   as the rest will use Prosody defaults.
///
/// See <https://prosody.im/doc/configure>.
#[derive(Debug)]
pub struct ProsodyConfig {
    pub log: LogConfig,
    pub admins: HashSet<JID>,
    pub enabled_modules: HashSet<String>,
    pub disabled_modules: HashSet<String>,
    pub limits: HashMap<ConnectionType, ConnectionLimits>,
    pub limits_resolution: Option<u32>,
    pub archive_expires_after: Option<PossiblyInfinite<DurationDate>>,
}

impl Default for ProsodyConfig {
    fn default() -> Self {
        Self {
            log: Default::default(),
            admins: Default::default(),
            enabled_modules: vec!["limits".to_string()].into_iter().collect(),
            disabled_modules: Default::default(),
            limits: Default::default(),
            limits_resolution: Default::default(),
            archive_expires_after: Default::default(),
        }
    }
}

impl ToString for ProsodyConfig {
    fn to_string(&self) -> String {
        let mut file = format!(
            "-- This file has been automatically generated by Prose Pod API.
-- Do NOT edit this file manually or your changes will be overriden during the next reload.

log = {log}

admins = {{{admins}}}

enabled_modules = {{{enabled_modules}}}
disabled_modules = {{{disabled_modules}}}

limits = {{{limits}}}",
            log = self.log,
            admins = format_set(&self.admins),
            enabled_modules = format_set(&self.enabled_modules),
            disabled_modules = format_set(&self.disabled_modules),
            limits = format_map(&self.limits),
        );

        if let Some(limits_resolution) = self.limits_resolution {
            file.push_str(&format!("\nlimits_resolution = {limits_resolution}"));
        }

        if let Some(duration) = &self.archive_expires_after {
            file.push_str(&format!("\n\narchive_expires_after = {}", format_duration_date_inf(duration)));
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

fn format_map<K, V>(map: &HashMap<K, V>) -> String
where
    K: Display,
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
fn format_duration_date(duration: &DurationDate) -> String {
    match duration {
        DurationDate::Days(n) => format!("{n}d"),
        DurationDate::Weeks(n) => format!("{n}w"),
        DurationDate::Months(n) => format!("{n}m"),
        DurationDate::Years(n) => format!("{n}y"),
    }
}

/// Format defined in <https://prosody.im/doc/modules/mod_mam#archive_expiry>.
fn format_duration_date_inf(duration: &PossiblyInfinite<DurationDate>) -> String {
    match duration {
        PossiblyInfinite::Infinite => "never".to_string(),
        PossiblyInfinite::Finite(duration) => format_duration_date(duration),
    }
}

/// See <https://prosody.im/doc/logging>.
#[derive(Debug)]
pub enum LogConfig {
    /// One value (file path, `"*syslog"` or "*console")
    Raw(LogLevelValue),
    Map(HashMap<LogLevel, LogLevelValue>),
}

impl Display for LogConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(v) => v.fmt(f),
            Self::Map(map) => format_map(map).fmt(f),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::Raw(LogLevelValue::FilePath("prosody.log".to_string()))
    }
}

/// See <https://prosody.im/doc/logging>.
#[derive(Debug)]
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

impl Display for LogLevelValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FilePath(p) => p.fmt(f),
            Self::Console => write!(f, "*console"),
            Self::Syslog => write!(f, "*syslog"),
        }
    }
}

#[derive(Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
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

/// Values from <https://prosody.im/doc/modules/mod_limits>.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ConnectionType {
    /// "c2s"
    ClientToServer,
    /// "s2sin"
    ServerToServerInbounds,
    /// "s2sout"
    ServerToServerOutbounds,
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

impl Into<ConnectionType> for crate::server_ctl::ConnectionType {
    fn into(self) -> ConnectionType {
        match self {
            Self::ClientToServer => ConnectionType::ClientToServer,
            Self::ServerToServerInbounds => ConnectionType::ServerToServerInbounds,
            Self::ServerToServerOutbounds => ConnectionType::ServerToServerOutbounds,
        }
    }
}

/// See <https://prosody.im/doc/modules/mod_limits>.
#[derive(Debug, Default)]
pub struct ConnectionLimits {
    pub rate: Option<DataRate>,
    pub burst: Option<DurationTime>,
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
        let mut map: HashMap<&str, String> = HashMap::new();

        if let Some(rate) = &self.rate {
            map.insert("rate", format_data_rate(rate));
        }
        if let Some(burst) = &self.burst {
            map.insert("burst", format!("{}s", burst.seconds()));
        }

        format_map(&map).fmt(f)
    }
}
