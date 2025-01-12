// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_set::LinkedHashSet;
use serde::{Deserialize, Serialize};

use crate::models::{durations::*, JidDomain};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub domain: JidDomain,
    pub message_archive_enabled: bool,
    pub message_archive_retention: PossiblyInfinite<Duration<DateLike>>,
    pub file_upload_allowed: bool,
    pub file_storage_encryption_scheme: String,
    pub file_storage_retention: PossiblyInfinite<Duration<DateLike>>,
    pub mfa_required: bool,
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS>.
    pub tls_profile: TlsProfile,
    pub federation_enabled: bool,
    pub federation_whitelist_enabled: bool,
    pub federation_friendly_servers: LinkedHashSet<String>,
    pub settings_backup_interval: String,
    pub user_data_backup_interval: String,
    pub push_notification_with_body: bool,
    pub push_notification_with_sender: bool,
}

/// See <https://wiki.mozilla.org/Security/Server_Side_TLS>.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(strum::EnumIter, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
pub enum TlsProfile {
    /// Modern clients that support TLS 1.3, with no need for backwards compatibility.
    ///
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS#Modern_compatibility>.
    Modern,
    /// Recommended configuration for a general-purpose server.
    ///
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS#Intermediate_compatibility_(recommended)>.
    Intermediate,
    /// Services accessed by very old clients or libraries, such as Internet Explorer 8 (Windows XP), Java 6, or OpenSSL 0.9.8.
    ///
    /// See <https://wiki.mozilla.org/Security/Server_Side_TLS#Old_backward_compatibility>.
    Old,
}
