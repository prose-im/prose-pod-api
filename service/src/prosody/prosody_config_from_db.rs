// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use entity::model as db;
use entity::server_config::Model as ServerConfig;
use prosody_config::{linked_hash_set::LinkedHashSet, *};
use utils::def;

use crate::config::Config;
use crate::ProseDefault;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProsodyConfig(prosody_config::ProsodyConfig);

impl Deref for ProsodyConfig {
    type Target = prosody_config::ProsodyConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ProsodyConfig {
    pub fn print(&self) -> ProsodyConfigFile {
        self.0.clone().print(vec![
            "Prose Pod Server".into(),
            "XMPP Server Configuration".into(),
            r#"/!\ This file has been automatically generated by Prose Pod API."#.into(),
            r#"/!\ Do NOT edit this file manually or your changes will be overriden during the next reload."#.into(),
        ].into())
    }

    pub fn all_sections(&self) -> Vec<&ProsodySettings> {
        let mut all_sections = vec![&self.global_settings];
        all_sections.extend(
            self.additional_sections
                .iter()
                .map(|section| section.settings()),
        );
        all_sections
    }
    pub fn all_enabled_modules(&self) -> LinkedHashSet<String> {
        self.all_sections()
            .iter()
            .map(|section| section.modules_enabled.clone().unwrap_or_default())
            .reduce(|acc, e| acc.union(&e).cloned().collect())
            .unwrap_or_default()
    }

    pub fn component_settings(&self, name: &str) -> Option<&ProsodySettings> {
        self.additional_sections
            .iter()
            .find_map(|section| match section {
                ProsodyConfigSection::Component {
                    plugin, settings, ..
                } if plugin.as_str() == name => Some(settings),
                _ => None,
            })
    }
}

impl ToString for ProsodyConfig {
    fn to_string(&self) -> String {
        self.print().to_string()
    }
}

pub fn prosody_config_from_db(model: ServerConfig, app_config: &Config) -> ProsodyConfig {
    let mut config = prosody_config::ProsodyConfig::prose_default(&model, app_config);

    let global_settings = &mut config.global_settings;
    let muc_settings = config
        .additional_sections
        .iter_mut()
        .find_map(|section| match section {
            ProsodyConfigSection::Component {
                plugin, settings, ..
            } if plugin.as_str() == "muc" => Some(settings),
            _ => None,
        })
        .expect("The 'Chatrooms' section should always be present.");

    if model.message_archive_enabled {
        add_enabled_module(global_settings, "mam");
        global_settings.archive_expires_after =
            Some(model.message_archive_retention.into_prosody());
        global_settings.default_archive_policy = Some(ArchivePolicy::Always);
        global_settings.max_archive_query_results = Some(100);
        add_enabled_module(muc_settings, "muc_mam");
    }

    if model.file_upload_allowed {
        config
            .additional_sections
            .push(ProsodyConfigSection::Component {
                hostname: "upload.prose.org.local".into(),
                plugin: "http_file_share".into(),
                name: "HTTP File Upload".into(),
                settings: ProsodySettings {
                    http_file_share_size_limit: Some(Bytes::MebiBytes(20)),
                    http_file_share_daily_quota: Some(Bytes::MebiBytes(250)),
                    http_file_share_expires_after: Some(
                        model.file_storage_retention.into_prosody(),
                    ),
                    http_host: Some("localhost".into()),
                    http_external_url: Some("http://localhost:5280".into()),
                    ..Default::default()
                },
            })
    }

    ProsodyConfig(config)
}

// ===== Default configuration =====

impl ProseDefault for prosody_config::ProsodyConfig {
    fn prose_default(server_config: &ServerConfig, app_config: &Config) -> Self {
        let api_jid = app_config.api_jid();
        let api_jid = JID::try_from(api_jid.to_string()).expect("Invalid JID: {api_jid}");
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
                http_ports: Some(vec![app_config.server.http_port]),
                http_interfaces: Some(vec![Interface::AllIPv4]),
                https_ports: Some(vec![]),
                https_interfaces: Some(vec![]),
                modules_enabled: Some(
                    vec![
                        "auto_activate_hosts",
                        "roster",
                        "groups_internal",
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
                cross_domain_websocket: None,
                contact_info: Some(ContactInfo {
                    admin: vec!["mailto:hostmaster@prose.org.local".to_string()],
                    ..Default::default()
                }),
                upgrade_legacy_vcards: Some(true),
                ..Default::default()
            },
            additional_sections: vec![
                ProsodyConfigSection::VirtualHost {
                    hostname: server_config.domain.to_owned(),
                    settings: ProsodySettings {
                        admins: Some(vec![api_jid.to_owned()].into_iter().collect()),
                        modules_enabled: Some(
                            vec![
                                "rest",
                                "http_oauth2",
                                "admin_rest",
                                // "init_admin",
                            ]
                            .into_iter()
                            .map(ToString::to_string)
                            .collect(),
                        ),
                        http_host: Some(app_config.server.local_hostname.to_owned()),
                        custom_settings: vec![
                            Group::new(
                                "mod_http_oauth2",
                                vec![def(
                                    "allowed_oauth2_grant_types",
                                    vec!["password"],
                                )],
                            ),
                            // // See <https://github.com/prose-im/prose-pod-server/blob/3b54d071880dff669f0193a8068733b089936751/plugins/mod_init_admin.lua>.
                            // Group::new(
                            //     "mod_init_admin",
                            //     vec![def(
                            //         "init_admin_jid",
                            //         api_jid.to_owned(),
                            //     )],
                            // ),
                        ],
                        ..Default::default()
                    },
                },
                ProsodyConfigSection::VirtualHost {
                    hostname: "admin.prose.org.local".into(),
                    settings: ProsodySettings {
                        admins: Some(vec![api_jid.to_owned()].into_iter().collect()),
                        modules_enabled: Some(
                            vec![
                                "admin_rest",
                                "init_admin",
                            ]
                            .into_iter()
                            .map(ToString::to_string)
                            .collect(),
                        ),
                        http_host: Some(app_config.server.local_hostname_admin.to_owned()),
                        custom_settings: vec![
                            // See <https://github.com/prose-im/prose-pod-server/blob/3b54d071880dff669f0193a8068733b089936751/plugins/mod_init_admin.lua>.
                            Group::new(
                                "mod_init_admin",
                                vec![
                                    def("init_admin_jid", api_jid.to_owned()),
                                    def(
                                        "init_admin_password_env_var_name",
                                        "PROSE_API__ADMIN_PASSWORD",
                                    ),
                                ],
                            ),
                        ],
                        ..Default::default()
                    },
                },
                ProsodyConfigSection::Component {
                    hostname: format!("groups.{}", server_config.domain),
                    plugin: "muc".into(),
                    name: "Chatrooms".into(),
                    settings: ProsodySettings {
                        restrict_room_creation: Some(RoomCreationRestriction::DomainOnly),
                        max_archive_query_results: Some(100),
                        muc_log_all_rooms: Some(true),
                        muc_log_expires_after: Some(PossiblyInfinite::Infinite),
                        muc_log_by_default: Some(true),
                        ..Default::default()
                    },
                },
            ],
        }
    }
}

// ===== Convenience methods =====

// TODO: Remove unused code

// fn add_admin(settings: &mut ProsodySettings, jid: JID) {
//     settings
//         .admins
//         .get_or_insert_with(Default::default)
//         .insert(jid);
// }
// fn remove_admin(settings: &mut ProsodySettings, jid: &JID) {
//     settings
//         .admins
//         .get_or_insert_with(Default::default)
//         .remove(jid);
// }

fn add_enabled_module(settings: &mut ProsodySettings, module_name: &'static str) -> bool {
    settings
        .modules_enabled
        .get_or_insert_with(Default::default)
        .insert(module_name.into())
}
// fn remove_enabled_module(settings: &mut ProsodySettings, module_name: &String) -> bool {
//     settings
//         .modules_enabled
//         .get_or_insert_with(Default::default)
//         .remove(module_name)
// }

// fn add_disabled_module(settings: &mut ProsodySettings, module_name: String) -> bool {
//     settings
//         .modules_disabled
//         .get_or_insert_with(Default::default)
//         .insert(module_name)
// }
// fn remove_disabled_module(settings: &mut ProsodySettings, module_name: &String) -> bool {
//     settings
//         .modules_disabled
//         .get_or_insert_with(Default::default)
//         .remove(module_name)
// }

// fn set_rate_limit(settings: &mut ProsodySettings, conn_type: ConnectionType, value: DataRate) {
//     settings
//         .limits
//         .get_or_insert_with(Default::default)
//         .entry(conn_type.into())
//         .or_insert_with(Default::default)
//         .rate = Some(value)
// }
// fn set_burst_limit(
//     settings: &mut ProsodySettings,
//     conn_type: ConnectionType,
//     value: Duration<TimeLike>,
// ) {
//     settings
//         .limits
//         .get_or_insert_with(Default::default)
//         .entry(conn_type.into())
//         .or_insert_with(Default::default)
//         .burst = Some(value)
// }

// ===== Model mappings =====

pub trait IntoProsody<T> {
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
