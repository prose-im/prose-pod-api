// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use prosody_config::*;

#[test]
fn test_serializing_enums() -> Result<(), serde_json::Error> {
    let storage = StorageConfig::Raw(StorageBackend::SQL);
    assert_eq!(serde_json::to_string(&storage)?, r#""sql""#);

    let storage = StorageConfig::Map(
        [("roster".to_owned(), StorageBackend::SQL)]
            .into_iter()
            .collect(),
    );
    assert_eq!(serde_json::to_string(&storage)?, r#"{"roster":"sql"}"#);

    let interface = Interface::Address("127.0.0.1".to_owned());
    assert_eq!(serde_json::to_string(&interface)?, r#""127.0.0.1""#);

    Ok(())
}

#[test]
fn test_serializing_prosody_config() -> Result<(), serde_json::Error> {
    let config = ProsodyConfig {
        global_settings: ProsodySettings {
            pidfile: Some("/var/run/prosody/prosody.pid".into()),
            admins: Some(
                vec![JID::from_str("remi@prose.org.local").unwrap()]
                    .into_iter()
                    .collect(),
            ),
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
            interfaces: Some(vec![Interface::AllIPv4]),
            c2s_ports: Some(vec![5222]),
            http_interfaces: Some(vec![Interface::AllIPv4]),
            plugin_paths: None,
            modules_enabled: None,
            modules_disabled: None,
            ssl: None,
            allow_registration: None,
            c2s_require_encryption: None,
            s2s_require_encryption: None,
            s2s_secure_auth: None,
            c2s_stanza_size_limit: None,
            s2s_stanza_size_limit: None,
            s2s_whitelist: None,
            limits: None,
            consider_websocket_secure: None,
            cross_domain_websocket: None,
            contact_info: None,
            archive_expires_after: None,
            default_archive_policy: None,
            max_archive_query_results: None,
            upgrade_legacy_vcards: None,
            groups_file: None,
            http_file_share_size_limit: None,
            http_file_share_daily_quota: None,
            http_file_share_expires_after: None,
            http_host: None,
            http_external_url: None,
            restrict_room_creation: None,
            muc_log_all_rooms: None,
            muc_log_by_default: None,
            muc_log_expires_after: None,
            tls_profile: None,
            custom_settings: vec![],
            ..Default::default()
        },
        additional_sections: vec![
            ProsodyConfigSection::VirtualHost {
                hostname: "example.org".to_owned(),
                settings: ProsodySettings {
                    pidfile: None,
                    admins: None,
                    authentication: None,
                    storage: None,
                    log: None,
                    interfaces: None,
                    c2s_ports: None,
                    s2s_ports: None,
                    http_ports: None,
                    http_interfaces: None,
                    https_ports: None,
                    https_interfaces: None,
                    plugin_paths: None,
                    modules_enabled: None,
                    modules_disabled: None,
                    ssl: None,
                    allow_registration: None,
                    c2s_require_encryption: None,
                    s2s_require_encryption: None,
                    s2s_secure_auth: None,
                    c2s_stanza_size_limit: None,
                    s2s_stanza_size_limit: None,
                    s2s_whitelist: None,
                    limits: None,
                    consider_websocket_secure: None,
                    cross_domain_websocket: None,
                    contact_info: None,
                    archive_expires_after: None,
                    default_archive_policy: None,
                    max_archive_query_results: None,
                    upgrade_legacy_vcards: None,
                    groups_file: None,
                    http_file_share_size_limit: None,
                    http_file_share_daily_quota: None,
                    http_file_share_expires_after: None,
                    http_host: None,
                    http_external_url: None,
                    restrict_room_creation: None,
                    muc_log_all_rooms: None,
                    muc_log_by_default: None,
                    muc_log_expires_after: None,
                    tls_profile: None,
                    custom_settings: vec![],
                },
            },
            ProsodyConfigSection::Component {
                hostname: "groups.prose.org.local".into(),
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
        ],
    };

    let config_json = serde_json::to_string_pretty(&config)?;
    insta::assert_snapshot!(config_json);

    Ok(())
}
