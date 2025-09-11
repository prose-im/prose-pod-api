// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    str::FromStr as _,
};

use linked_hash_set::LinkedHashSet;
use rand::RngCore as _;
use secrecy::SecretString;

use crate::{
    invitations::InvitationChannel,
    models::{DateLike, Duration, JidNode, PossiblyInfinite, TimeLike},
    server_config::TlsProfile,
};

use super::*;

// GENERAL

#[cfg(debug_assertions)]
pub fn true_in_debug() -> bool {
    true
}
#[cfg(not(debug_assertions))]
pub fn true_in_debug() -> bool {
    false
}

pub fn always_true() -> bool {
    true
}
pub fn always_false() -> bool {
    false
}

// SPECIFIC

pub fn log_level() -> LogLevel {
    LogLevel::Info
}

pub fn log_format() -> LogFormat {
    if cfg!(debug_assertions) {
        LogFormat::Pretty
    } else {
        LogFormat::Json
    }
}

pub fn log_timer() -> LogTimer {
    if cfg!(debug_assertions) {
        LogTimer::Uptime
    } else {
        LogTimer::Time
    }
}

pub fn bootstrap_prose_pod_api_xmpp_password() -> SecretString {
    "bootstrap".into()
}

pub fn service_accounts_prose_pod_api() -> ServiceAccountConfig {
    ServiceAccountConfig {
        xmpp_node: JidNode::from_str("prose-pod-api").unwrap(),
    }
}

pub fn service_accounts_prose_workspace() -> ServiceAccountConfig {
    ServiceAccountConfig {
        xmpp_node: JidNode::from_str("prose-workspace").unwrap(),
    }
}

pub fn workspace_xmpp_node() -> JidNode {
    JidNode::from_str("prose-workspace").expect("Invalid default `workspace_xmpp_node`")
}

pub fn server_local_hostname() -> String {
    "prose-pod-server".to_string()
}

pub fn server_local_hostname_admin() -> String {
    "prose-pod-server-admin".to_string()
}

pub fn server_http_port() -> u16 {
    5280
}

/// 3 hours.
pub fn auth_token_ttl() -> iso8601_duration::Duration {
    iso8601_duration::Duration::new(0., 0., 0., 3., 0., 0.)
}

/// 15 minutes.
pub fn auth_password_reset_token_ttl() -> iso8601_duration::Duration {
    iso8601_duration::Duration::new(0., 0., 0., 0., 15., 0.)
}

pub fn auth_oauth2_registration_key() -> SecretString {
    let mut key = [0u8; 256];
    rand::thread_rng().fill_bytes(&mut key);

    // fn bytes_to_hex(bytes: &[u8]) -> String {
    //     bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
    // }
    fn bytes_to_base64(bytes: &[u8]) -> String {
        use base64::Engine as _;
        base64::prelude::BASE64_STANDARD.encode(bytes)
    }

    SecretString::from(bytes_to_base64(&key))
}

pub fn server_log_level() -> prosody_config::LogLevel {
    prosody_config::LogLevel::Info
}

/// Enable message archiving by default.
pub fn server_defaults_message_archive_enabled() -> bool {
    true
}

/// Keep indefinitely, as defined in <https://github.com/prose-im/prose-pod-system/blob/f2e353758e628c01c0923fc0e46491f1644354c9/server/etc/prosody/prosody.cfg.lua#L97>.
pub fn server_defaults_message_archive_retention() -> PossiblyInfinite<Duration<DateLike>> {
    PossiblyInfinite::Infinite
}

/// Enable file upload by default.
pub fn server_defaults_file_upload_allowed() -> bool {
    true
}

// TODO: Make `FileStorageEncryptionScheme` an enum
/// Encrypt files in [AES 256](https://fr.wikipedia.org/wiki/Advanced_Encryption_Standard) by default.
pub fn server_defaults_file_storage_encryption_scheme() -> String {
    "AES-256".to_string()
}

/// Keep indefinitely, as defined in <https://github.com/prose-im/prose-pod-system/blob/f2e353758e628c01c0923fc0e46491f1644354c9/server/etc/prosody/prosody.cfg.lua#L126>.
pub fn server_defaults_file_storage_retention() -> PossiblyInfinite<Duration<DateLike>> {
    PossiblyInfinite::Infinite
}

/// Enable MFA by default.
pub fn server_defaults_mfa_required() -> bool {
    true
}

/// Default minimum [TLS](https://fr.wikipedia.org/wiki/Transport_Layer_Security) profile
/// (see <https://wiki.mozilla.org/Security/Server_Side_TLS>).
pub fn server_defaults_tls_profile() -> TlsProfile {
    TlsProfile::Modern
}

/// Disable federation by default.
pub fn server_defaults_federation_enabled() -> bool {
    false
}

/// Federate with the whole XMPP network by default.
pub fn server_defaults_federation_whitelist_enabled() -> bool {
    false
}

/// Do not trust any other server by default.
pub fn server_defaults_federation_friendly_servers() -> LinkedHashSet<JidDomain> {
    LinkedHashSet::default()
}

/// 1 day in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601#Durations).
pub fn server_defaults_settings_backup_interval() -> String {
    "P1D".to_string()
}

/// 1 week in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601#Durations).
pub fn server_defaults_user_data_backup_interval() -> String {
    "P1W".to_string()
}

pub fn server_defaults_push_notification_with_body() -> bool {
    true
}

pub fn server_defaults_push_notification_with_sender() -> bool {
    true
}

pub fn prosody_config_file_path() -> PathBuf {
    PathBuf::from("/etc/prosody/prosody.cfg.lua")
}

pub fn branding_api_app_name() -> String {
    "Prose Pod API".to_string()
}

pub fn notify_workspace_invitation_channel() -> InvitationChannel {
    InvitationChannel::Email
}

pub fn smtp_port() -> u16 {
    587
}

pub fn smtp_encrypt() -> bool {
    true
}

pub fn databases_main() -> DatabaseConfig {
    DatabaseConfig {
        url: format!("sqlite://{API_DATA_DIR}/database.sqlite"),
        min_connections: Default::default(),
        max_connections: database_max_connections(),
        connect_timeout: database_connect_timeout(),
        idle_timeout: Default::default(),
        sqlx_logging: Default::default(),
    }
}

pub fn database_max_connections() -> usize {
    let workers: usize =
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    workers * 4
}

pub fn database_connect_timeout() -> u64 {
    5
}

pub fn api_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

pub fn api_port() -> u16 {
    8080
}

/// 10 seconds seems reasonable, as it's enough to go around the globe multiple times.
pub fn api_default_response_timeout() -> Duration<TimeLike> {
    Duration(TimeLike::Seconds(10))
}

pub fn api_default_retry_interval() -> Duration<TimeLike> {
    Duration(TimeLike::Seconds(5))
}
