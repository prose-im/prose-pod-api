// prose-pod-api
//
// Copyright:
//   - 2018, Valerian Saliou <valerian@valeriansaliou.name> via valeriansaliou/vigil
//   - 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{path::PathBuf, str::FromStr as _};

use crate::model::{DateLike, Duration, InvitationChannel, JidNode, PossiblyInfinite};

pub fn api_admin_node() -> JidNode {
    JidNode::from_str("prose-pod-api").expect("Invalid default `api_admin_node`")
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

pub fn server_prosody_config_file_path() -> PathBuf {
    PathBuf::from("/etc/prosody/prosody.cfg.lua")
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

// TODO: Make `MinimumTLSVersion` an enum
/// Default minimum [TLS](https://fr.wikipedia.org/wiki/Transport_Layer_Security) version.
pub fn server_defaults_minimum_tls_version() -> String {
    "1.2".to_string()
}

// TODO: Make `MinimumCipherSuite` an enum
/// High security by default.
pub fn server_defaults_minimum_cipher_suite() -> String {
    "HIGH_STRENGTH".to_string()
}

/// Enable federation by default.
pub fn server_defaults_federation_enabled() -> bool {
    true
}

/// 1 day in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601#Durations).
pub fn server_defaults_settings_backup_interval() -> String {
    "P1D".to_string()
}

/// 1 week in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601#Durations).
pub fn server_defaults_user_data_backup_interval() -> String {
    "P1W".to_string()
}

pub fn branding_page_title() -> String {
    "Prose Pod API".to_string()
}

pub fn notify_workspace_invitation_channel() -> InvitationChannel {
    InvitationChannel::Email
}

pub fn notify_email_smtp_host() -> String {
    "localhost".to_string()
}

pub fn notify_email_smtp_port() -> u16 {
    587
}

pub fn notify_email_smtp_encrypt() -> bool {
    true
}
