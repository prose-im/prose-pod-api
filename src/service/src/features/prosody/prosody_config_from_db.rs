// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prosody_config::{utils::def, *};
use tracing::{info, warn};

use crate::{
    app_config::{AppConfig, FILE_SHARE_HOST},
    models::{self as db},
    ProseDefault, ServerConfig,
};

use super::{prosody_config::util::*, ProsodyConfig};

pub fn prosody_config_from_db(model: ServerConfig, app_config: &AppConfig) -> ProsodyConfig {
    let mut config = prosody_config::ProsodyConfig::prose_default(&model, app_config);

    // NOTE: Deconstruct to ensure no value is unused.
    let ServerConfig {
        domain,
        message_archive_enabled,
        message_archive_retention,
        file_upload_allowed,
        file_storage_encryption_scheme: _file_storage_encryption_scheme,
        file_storage_retention,
        mfa_required: _mfa_required,
        tls_profile: _tls_profile,
        federation_enabled,
        federation_whitelist_enabled,
        federation_friendly_servers,
        settings_backup_interval: _settings_backup_interval,
        user_data_backup_interval: _user_data_backup_interval,
        push_notification_with_body,
        push_notification_with_sender,
        c2s_unencrypted,
        prosody_overrides,
        // NOTE: Used in `live_server_ctl::save_config`.
        prosody_overrides_raw: _,
    } = model;

    let global_settings = &mut config.global_settings;
    let muc_settings = (config.additional_sections)
        .iter_mut()
        .find_map(|section| match section {
            ProsodyConfigSection::Component {
                plugin, settings, ..
            } if plugin.as_str() == "muc" => Some(settings),
            _ => None,
        })
        .expect("The 'Chatrooms' section should always be present.");

    global_settings.custom_settings.push(
        // See <https://modules.prosody.im/mod_cloud_notify>
        Group::new(
            "mod_cloud_notify",
            vec![
                def("push_notification_with_body", push_notification_with_body),
                def(
                    "push_notification_with_sender",
                    push_notification_with_sender,
                ),
            ],
        ),
    );

    if message_archive_enabled {
        add_enabled_module(global_settings, "mam");
        global_settings.archive_expires_after = Some(message_archive_retention.into_prosody());
        global_settings.default_archive_policy = Some(ArchivePolicy::Always);
        global_settings.max_archive_query_results = Some(100);
        add_enabled_module(muc_settings, "muc_mam");
    }

    if file_upload_allowed {
        let host = FILE_SHARE_HOST;
        let main_host_settings = (config.additional_sections)
            .iter_mut()
            .find_map(|section| match section {
                ProsodyConfigSection::VirtualHost {
                    hostname, settings, ..
                } if *hostname == domain.to_string() => Some(settings),
                _ => None,
            })
            .expect(&format!(
                "The '{domain}' virtual host should always be present."
            ));
        (main_host_settings.disco_items.get_or_insert_default()).insert(DiscoItem {
            address: host.to_owned(),
            name: "HTTP File Upload".to_owned(),
        });
        (config.additional_sections).push(ProsodyConfigSection::Component {
            hostname: host.to_owned(),
            plugin: "http_file_share".to_owned(),
            name: "HTTP File Upload".to_owned(),
            settings: ProsodySettings {
                http_file_share_size_limit: Some(Bytes::MebiBytes(20)),
                http_file_share_daily_quota: Some(Bytes::MebiBytes(100)),
                http_file_share_expires_after: Some(file_storage_retention.into_prosody()),
                http_file_share_access: Some(
                    vec![BareJid::domain(
                        domain,
                    )]
                    .into_iter()
                    .collect(),
                ),
                http_external_url: Some(app_config.app_web_url().to_string()),
                http_paths: Some(
                    vec![("file_share".to_owned(), "/upload".to_owned())]
                        .into_iter()
                        .collect(),
                ),
                ..Default::default()
            }
            .with_defaults_and_overrides_from(app_config, host),
        })
    }

    if federation_enabled {
        global_settings.enable_module("s2s".to_owned());
        global_settings.enable_module("s2s_bidi".to_owned());
        global_settings.s2s_ports = Some(vec![5269].into_iter().collect());
        global_settings.s2s_require_encryption = Some(true);
        global_settings.s2s_secure_auth = Some(false);
        global_settings.limits.get_or_insert_default().insert(
            ConnectionType::ServerToServerInbounds,
            ConnectionLimits {
                rate: Some(DataRate::KiloBytesPerSec(250)),
                burst: Some(Duration(TimeLike::Seconds(4))),
            },
        );

        if federation_whitelist_enabled {
            global_settings.enable_module("s2s_whitelist".to_owned());
            global_settings.s2s_whitelist = Some(federation_friendly_servers);
        }
    }

    if c2s_unencrypted {
        warn!("Debug config `c2s_unencrypted` is enabled.");
        global_settings.enable_module("reload_modules".to_owned());
        (global_settings.reload_modules.get_or_insert_default()).insert("saslauth".to_owned());
        global_settings.custom_settings.push(Group::new(
            "Debug config: c2s_unencrypted",
            vec![
                def("c2s_require_encryption", false),
                def("allow_unencrypted_plain_auth", true),
            ],
        ));
    }

    for module in app_config.prosody_ext.additional_modules_enabled.iter() {
        config.global_settings.enable_module(module.clone());
    }

    if let Some(overrides) = prosody_overrides {
        info!("Applying overrides to the generated Prosody configuration file…");
        (config.global_settings).shallow_merge(overrides, MergeStrategy::KeepOther);
    }

    ProsodyConfig(config)
}

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
            Self::Seconds(n) => TimeLike::Seconds(n),
            Self::Minutes(n) => TimeLike::Minutes(n),
            Self::Hours(n) => TimeLike::Hours(n),
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

impl IntoProsody<ConnectionType> for db::xmpp::XmppDirectionalConnectionType {
    fn into_prosody(self) -> ConnectionType {
        match self {
            Self::ClientToServer => ConnectionType::ClientToServer,
            Self::ServerToServerInbounds => ConnectionType::ServerToServerInbounds,
            Self::ServerToServerOutbounds => ConnectionType::ServerToServerOutbounds,
        }
    }
}
