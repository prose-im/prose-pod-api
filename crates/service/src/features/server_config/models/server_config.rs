// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
    pub minimum_tls_version: String,
    pub minimum_cipher_suite: String,
    pub federation_enabled: bool,
    pub settings_backup_interval: String,
    pub user_data_backup_interval: String,
}
