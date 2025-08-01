// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prosody_config::{utils::def, *};

/// Value from <https://github.com/prose-im/prose-pod-system/blob/f2e353758e628c01c0923fc0e46491f1644354c9/server/etc/prosody/prosody.cfg.lua#L114>
/// (with slight modifications for style consistency)
#[test]
fn test_prose_default_config() {
    let api_jid = BareJid::new("prose-pod-api", "admin.prose.local");
    let default_config = ProsodyConfig {
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
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            )),
            interfaces: Some(vec![Interface::AllIPv4].into_iter().collect()),
            c2s_ports: Some(vec![5222].into_iter().collect()),
            s2s_ports: Some(vec![5269].into_iter().collect()),
            http_ports: Some(vec![5280].into_iter().collect()),
            http_interfaces: Some(vec![Interface::AllIPv4].into_iter().collect()),
            https_ports: Some(Default::default()),
            https_interfaces: Some(Default::default()),
            admins: Some(vec![api_jid.to_owned()].into_iter().collect()),
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
                .iter()
                .map(ToString::to_string)
                .collect(),
            ),
            ssl: Some(SSLConfig {
                certificate: Some("/etc/prosody/certs/prose.local.crt".into()),
                key: Some("/etc/prosody/certs/prose.local.key".into()),
                ..Default::default()
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
            contact_info: Some(ContactInfo {
                abuse: vec![],
                admin: vec!["mailto:hostmaster@prose.local".to_string()],
                feedback: vec![],
                sales: vec![],
                security: vec![],
                support: vec![],
            }),
            archive_expires_after: Some(PossiblyInfinite::Infinite),
            default_archive_policy: Some(ArchivePolicy::Always),
            max_archive_query_results: Some(100),
            upgrade_legacy_vcards: Some(true),
            groups_file: Some("/etc/prosody/roster_groups.txt".into()),
            http_host: Some("prose-pod-server".to_owned()),
            http_external_url: Some("http://prose-pod-server:5280".to_owned()),
            custom_settings: vec![Group::new(
                "mod_init_admin",
                vec![
                    def("init_admin_jid", api_jid.to_owned()),
                    def(
                        "init_admin_password_env_var_name",
                        "PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD",
                    ),
                ],
            )],
            ..Default::default()
        },
        additional_sections: vec![
            ProsodyConfigSection::VirtualHost {
                hostname: "admin.prose.local".into(),
                settings: ProsodySettings::default(),
            },
            ProsodyConfigSection::VirtualHost {
                hostname: "prose.local".into(),
                settings: ProsodySettings::default(),
            },
            ProsodyConfigSection::Component {
                hostname: "groups.prose.local".into(),
                plugin: "muc".into(),
                name: "Chatrooms".into(),
                settings: ProsodySettings {
                    modules_enabled: Some(
                        vec!["muc_mam"].iter().map(ToString::to_string).collect(),
                    ),
                    restrict_room_creation: Some(RoomCreationRestriction::DomainOnly),
                    muc_log_all_rooms: Some(true),
                    muc_log_by_default: Some(true),
                    muc_log_expires_after: Some(PossiblyInfinite::Infinite),
                    max_archive_query_results: Some(100),
                    ..Default::default()
                },
            },
            ProsodyConfigSection::Component {
                hostname: "upload.prose.local".into(),
                plugin: "http_file_share".into(),
                name: "HTTP File Upload".into(),
                settings: ProsodySettings {
                    http_file_share_size_limit: Some(Bytes::MebiBytes(20)),
                    http_file_share_daily_quota: Some(Bytes::MebiBytes(250)),
                    http_file_share_expires_after: Some(PossiblyInfinite::Infinite),
                    http_host: Some("localhost".into()),
                    http_external_url: Some("http://localhost:5280".into()),
                    ..Default::default()
                },
            },
        ],
    };

    let result = default_config.print(vec![
        "Prose Pod Server".into(),
        "XMPP Server Configuration".into(),
        r#"/!\ This file has been automatically generated by Prose Pod API."#.into(),
        r#"/!\ Do NOT edit this file manually or your changes will be overridden during the next reload."#.into(),
    ].into()).to_string();
    insta::assert_snapshot!(result);
}
