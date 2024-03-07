// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::model as db;
use entity::server_config::Model as ServerConfig;
use prosody_config::*;

use crate::ProseDefault;

pub fn prosody_config_from_db(model: ServerConfig) -> ProsodyConfig {
    let mut config = ProsodyConfig::prose_default();

    if model.message_archive_enabled {
        config
            .global_settings
            .modules_enabled
            .get_or_insert_with(Default::default)
            .insert("mam".to_string());
        config.global_settings.archive_expires_after =
            Some(model.message_archive_retention.into_prosody());
    }

    if model.file_upload_allowed {}

    config
}

// ===== Default configuration =====

impl ProseDefault for ProsodyConfig {
    fn prose_default() -> Self {
        Self {
            global_settings: ProsodySettings {
                pidfile: Some("/var/run/prosody/prosody.pid".into()),
                authentication: Some(AuthenticationProvider::InternalHashed),
                storage: Some(StorageConfig::Raw(StorageBackend::Internal)),
                log: Some(LogConfig::Map(
                    vec![
                        (LogLevel::Info, LogLevelValue::Console),
                        (LogLevel::Warn, LogLevelValue::Console),
                        (LogLevel::Error, LogLevelValue::Console),
                    ]
                    .into_iter()
                    .collect(),
                )),
                interfaces: Some(vec![Interface::AllIPv4]),
                c2s_ports: Some(vec![5222]),
                s2s_ports: Some(vec![5269]),
                http_ports: Some(vec![5280]),
                http_interfaces: Some(vec![Interface::AllIPv4]),
                https_ports: Some(vec![]),
                https_interfaces: Some(vec![]),
                admins: Some(Default::default()),
                modules_enabled: Some(
                    vec![
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
                ),
                ssl: Some(SSLConfig::Manual {
                    key: "/etc/prosody/certs/prose.org.local.key".into(),
                    certificate: "/etc/prosody/certs/prose.org.local.crt".into(),
                }),
                allow_registration: Some(false),
                c2s_require_encryption: Some(true),
                s2s_require_encryption: Some(true),
                s2s_secure_auth: Some(false),
                c2s_stanza_size_limit: Some(Bytes::KibiBytes(256)),
                s2s_stanza_size_limit: Some(Bytes::KibiBytes(512)),
                limits: Some(
                    vec![
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
                ),
                consider_websocket_secure: Some(true),
                cross_domain_websocket: Some(true),
                contact_info: Some(ContactInfo {
                    admin: vec!["mailto:hostmaster@prose.org.local".to_string()],
                    ..Default::default()
                }),
                archive_expires_after: Some(PossiblyInfinite::Infinite),
                default_archive_policy: Some(true),
                max_archive_query_results: Some(100),
                upgrade_legacy_vcards: Some(true),
                groups_file: Some("/etc/prosody/roster_groups.txt".into()),
                ..Default::default()
            },
            additional_sections: vec![],
        }
    }
}

// ===== Model mappings =====

trait IntoProsody<T> {
    fn into_prosody(self) -> T;
}

impl IntoProsody<DateLike> for db::DateLike {
    fn into_prosody(self) -> DateLike {
        match self {
            Self::Days(n) => DateLike::Days(n),
            Self::Weeks(n) => DateLike::Weeks(n),
            Self::Months(n) => DateLike::Months(n),
            Self::Years(n) => DateLike::Years(n),
        }
    }
}

impl IntoProsody<TimeLike> for db::TimeLike {
    fn into_prosody(self) -> TimeLike {
        match self {
            db::TimeLike::Seconds(_) => todo!(),
            db::TimeLike::Minutes(_) => todo!(),
            db::TimeLike::Hours(_) => todo!(),
        }
    }
}

impl<A, B> IntoProsody<Duration<B>> for db::Duration<A>
where
    A: db::DurationContent + IntoProsody<B>,
    B: DurationContent,
{
    fn into_prosody(self) -> Duration<B> {
        Duration(self.0.into_prosody())
    }
}

impl<A, B> IntoProsody<PossiblyInfinite<B>> for db::PossiblyInfinite<A>
where
    A: IntoProsody<B>,
{
    fn into_prosody(self) -> PossiblyInfinite<B> {
        match self {
            Self::Infinite => PossiblyInfinite::Infinite,
            Self::Finite(d) => PossiblyInfinite::Finite(d.into_prosody()),
        }
    }
}

impl IntoProsody<ConnectionType> for db::ConnectionType {
    fn into_prosody(self) -> ConnectionType {
        match self {
            Self::ClientToServer => ConnectionType::ClientToServer,
            Self::ServerToServerInbounds => ConnectionType::ServerToServerInbounds,
            Self::ServerToServerOutbounds => ConnectionType::ServerToServerOutbounds,
        }
    }
}

// Some old method definitions
// TODO: Remove useless code below

// fn add_admin(&mut self, jid: JID) {
//     self.config.admins.insert(jid);
// }
// fn remove_admin(&mut self, jid: &JID) {
//     self.config.admins.remove(jid);
// }

// fn add_enabled_module(&mut self, module_name: String) -> bool {
//     self.config.enabled_modules.insert(module_name)
// }
// fn remove_enabled_module(&mut self, module_name: &String) -> bool {
//     self.config.enabled_modules.remove(module_name)
// }

// fn add_disabled_module(&mut self, module_name: String) -> bool {
//     self.config.disabled_modules.insert(module_name)
// }
// fn remove_disabled_module(&mut self, module_name: &String) -> bool {
//     self.config.disabled_modules.remove(module_name)
// }

// fn set_rate_limit(&mut self, conn_type: ConnectionType, value: DataRate) {
//     self.config
//         .limits
//         .entry(conn_type.into())
//         .or_insert_with(Default::default)
//         .rate = Some(value)
// }
// fn set_burst_limit(&mut self, conn_type: ConnectionType, value: DurationTime) {
//     self.config
//         .limits
//         .entry(conn_type.into())
//         .or_insert_with(Default::default)
//         .burst = Some(value)
// }
// fn set_timeout(&mut self, value: DurationTime) {
//     self.config.limits_resolution = Some(value.seconds().clone());
// }

// fn enable_message_archiving(&mut self) -> bool {
//     self.add_enabled_module("mam".to_string())
// }
// fn disable_message_archiving(&mut self) -> bool {
//     self.remove_enabled_module(&"mam".to_string())
// }
