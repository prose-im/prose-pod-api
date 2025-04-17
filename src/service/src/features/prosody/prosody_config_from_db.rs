// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prosody_config::{utils::def, *};
use tracing::{info, warn};

use crate::{
    app_config::{AppConfig, FILE_SHARE_HOST},
    models::{self as db},
    server_config::ServerConfig,
    ProseDefault,
};

use super::{prosody_config::util::*, prosody_overrides::ProsodyOverrides, ProsodyConfig};

pub fn prosody_config_from_db(model: ServerConfig, app_config: &AppConfig) -> ProsodyConfig {
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
                hostname: FILE_SHARE_HOST.into(),
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

    if model.federation_enabled {
        global_settings.unmark_disabled("s2s");
        global_settings.enable_module("s2s_bidi".to_owned());
        global_settings.s2s_ports = Some(vec![5269]);
        global_settings.s2s_require_encryption = Some(true);
        global_settings.s2s_secure_auth = Some(false);
        global_settings.limits.get_or_insert_default().insert(
            ConnectionType::ServerToServerInbounds,
            ConnectionLimits {
                rate: Some(DataRate::KiloBytesPerSec(250)),
                burst: Some(Duration(TimeLike::Seconds(4))),
            },
        );

        if model.federation_whitelist_enabled {
            global_settings.enable_module("s2s_whitelist".to_owned());
            global_settings.s2s_whitelist = Some(model.federation_friendly_servers);
        }
    }

    if model.c2s_unencrypted {
        warn!("Debug config `c2s_unencrypted` is enabled.");
        global_settings.enable_module("reload_modules".to_owned());
        global_settings.custom_settings.push(Group::new(
            "Debug config: c2s_unencrypted",
            vec![
                def("c2s_require_encryption", false),
                def("allow_unencrypted_plain_auth", true),
                def("reload_modules", vec!["saslauth"]),
            ],
        ));
    }

    for module in app_config.prosody.additional_modules_enabled.iter() {
        config.global_settings.enable_module(module.clone());
    }

    if let Some(overrides) = model.prosody_overrides {
        match serde_json::from_value::<ProsodyOverrides>(overrides) {
            Ok(ProsodyOverrides {
                c2s_require_encryption,
            }) => {
                info!("Applying overrides to the generated Prosody configuration file…");
                macro_rules! override_global_setting {
                    ($var:ident) => {
                        if let Some($var) = $var {
                            config.global_settings.$var = Some($var);
                        }
                    };
                }
                override_global_setting!(c2s_require_encryption);
            },
            Err(err) => warn!("Prosody overrides stored in database cannot be read, they won’t be applied. To fix this, call `PUT /v1/server/config/prosody-overrides` with a new value. You can `GET /v1/server/config/prosody-overrides` first to see what the stored value was. Error: {err}"),
        }
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
