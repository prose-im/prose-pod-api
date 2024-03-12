// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prosody_config::utils::*;
use prosody_config::*;

/// Value from <https://github.com/prose-im/prose-pod-system/blob/f2e353758e628c01c0923fc0e46491f1644354c9/server/etc/prosody/prosody.cfg.lua#L114>
/// (with slight modifications for style consistency)
#[test]
fn test_prose_default_config() {
    let default_config = ProsodyConfigFile {
        header: Some(vec![
            "Prose Pod Server".into(),
            "XMPP Server Configuration".into(),
            r#"/!\ This file has been automatically generated by Prose Pod API."#.into(),
            r#"/!\ Do NOT edit this file manually or your changes will be overriden during the next reload."#.into(),
        ].into()),
        global_settings: vec![
            def("pidfile", "/var/run/prosody/prosody.pid").into(),
            vec![
                def("authentication", "internal_hashed"),
                def("storage", "internal"),
            ].into(),
            def(
                "log",
                vec![
                    ("info", "*console"),
                    ("warn", "*console"),
                    ("error", "*console"),
                ]
            ).into(),
            Group::new(
                "Network interfaces/ports",
                vec![
                    def("interfaces", vec!["*"]),
                    def("c2s_ports", vec![5222]),
                    def("s2s_ports", vec![5269]),
                    def("http_ports", vec![5280]),
                    def("http_interfaces", vec!["*"]),
                    def("https_ports", LuaValue::List(vec![])),
                    def("https_interfaces", LuaValue::List(vec![])),
                ],
            ),
            Group::new(
                "Modules",
                vec![
                    def("modules_enabled", vec![
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
                    ]),
                ],
            ),
            def("ssl", vec![
                ("certificate", "/etc/prosody/certs/prose.org.local.crt"),
                ("key", "/etc/prosody/certs/prose.org.local.key"),
            ])
            .comment("Path to SSL key and certificate for all server domains")
            .into(),
            def("allow_registration", false)
                .comment("Disable in-band registrations (done through the Prose Pod Dashboard/API)")
                .into(),
            Group::new(
                "Mandate highest security levels",
                vec![
                    def("c2s_require_encryption", true),
                    def("s2s_require_encryption", true),
                    def("s2s_secure_auth", false),
                ],
            ),
            Group::new(
                "Enforce safety C2S/S2S limits",
                vec![
                    def("c2s_stanza_size_limit", mult(256, 1024)),
                    def("s2s_stanza_size_limit", mult(512, 1024)),
                ],
            ),
            def("limits", vec![
                ("c2s", vec![
                    ("rate", "50kb/s"),
                    ("burst", "2s"),
                ]),
                ("s2sin", vec![
                    ("rate", "250kb/s"),
                    ("burst", "4s"),
                ]),
            ]).into(),
            Group::new(
                "Allow reverse-proxying to WebSocket service over insecure local HTTP",
                vec![
                    def("consider_websocket_secure", true),
                    def("cross_domain_websocket", true),
                ],
            ),
            def("contact_info", vec![
                ("admin", vec!["mailto:hostmaster@prose.org.local"]),
            ])
            .comment("Specify server administrator")
            .into(),
            Group::new(
                "MAM settings",
                vec![
                    def("archive_expires_after", "never"),
                    def("default_archive_policy", true),
                    def("max_archive_query_results", 100)
                ],
            ),
            def("upgrade_legacy_vcards", true)
                .comment("Enable vCard legacy compatibility layer")
                .into(),
            def("groups_file", "/etc/prosody/roster_groups.txt")
                .comment("Define server members groups file")
                .into(),
        ],
        additional_sections: vec![
            ProsodyConfigFileSection::VirtualHost {
                comments: vec![],
                hostname: "prose.org.local".to_string(),
                settings: vec![],
            },
            ProsodyConfigFileSection::Component {
                comments: vec![],
                hostname: "groups.prose.org.local".to_string(),
                plugin: "muc".to_string(),
                name: "Chatrooms".to_string(),
                settings: vec![

                    Group::new(
                        "Modules",
                        vec![
                            def("modules_enabled", vec!["muc_mam"])
                        ],
                    ),
                    def("restrict_room_creation", "local").into(),
                    vec![
                        def("log_all_rooms", true),
                        def("muc_log_expires_after", "never"),
                    ].into(),
                ],
            },
            ProsodyConfigFileSection::Component {
                comments: vec![],
                hostname: "upload.prose.org.local".to_string(),
                plugin: "http_file_share".to_string(),
                name: "HTTP File Upload".to_string(),
                settings: vec![
                    vec![
                        def("http_file_share_size_limit", mult(20, mult(1024, 1024))),
                        def("http_file_share_daily_quota", mult(250, mult(1024, 1024))),
                        def("http_file_share_expires_after", -1),
                        def("http_host", "localhost"),
                        def("http_external_url", "http://localhost:5280/"),
                    ].into(),
                ],
            },
        ],
    };

    let result = default_config.to_string();
    insta::assert_snapshot!(result);
}
