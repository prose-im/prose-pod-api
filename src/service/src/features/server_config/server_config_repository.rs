// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prosody_config::ProsodySettings;

use crate::{
    models::{durations::*, sea_orm::LinkedStringSet, Lua},
    server_config::TlsProfile,
};

use super::DynamicServerConfig;

crate::gen_scoped_kv_store!(pub(super) server_config);

crate::gen_kv_store_scoped_get_set!(pub message_archive_enabled: bool [+delete]);
crate::gen_kv_store_scoped_get_set!(pub message_archive_retention: PossiblyInfinite<Duration<DateLike>> [+delete]);
crate::gen_kv_store_scoped_get_set!(pub file_upload_allowed: bool [+delete]);
crate::gen_kv_store_scoped_get_set!(pub file_storage_encryption_scheme: String [+delete]);
crate::gen_kv_store_scoped_get_set!(pub file_storage_retention: PossiblyInfinite<Duration<DateLike>> [+delete]);
crate::gen_kv_store_scoped_get_set!(pub mfa_required: bool [+delete]);
crate::gen_kv_store_scoped_get_set!(pub tls_profile: TlsProfile [+delete]);
crate::gen_kv_store_scoped_get_set!(pub federation_enabled: bool [+delete]);
crate::gen_kv_store_scoped_get_set!(pub federation_whitelist_enabled: bool [+delete]);
crate::gen_kv_store_scoped_get_set!(pub federation_friendly_servers: LinkedStringSet [+delete]);
crate::gen_kv_store_scoped_get_set!(pub settings_backup_interval: String [+delete]);
crate::gen_kv_store_scoped_get_set!(pub user_data_backup_interval: String [+delete]);
crate::gen_kv_store_scoped_get_set!(pub push_notification_with_body: bool [+delete]);
crate::gen_kv_store_scoped_get_set!(pub push_notification_with_sender: bool [+delete]);
crate::gen_kv_store_scoped_get_set!(pub prosody_overrides: ProsodySettings [+delete] [+default]);
crate::gen_kv_store_scoped_get_set!(pub prosody_overrides_raw: Lua [+delete]);

pub async fn get(db: &impl sea_orm::ConnectionTrait) -> anyhow::Result<DynamicServerConfig> {
    use anyhow::Context;
    let kv = self::kv_store::get_all(db).await?;
    let json = serde_json::Value::Object(kv);
    serde_json::from_value::<DynamicServerConfig>(json)
        .context("Could not parse `DynamicServerConfig` from key/value data.")
}

pub async fn reset(db: &impl sea_orm::ConnectionTrait) -> anyhow::Result<()> {
    match self::kv_store::delete_all(db).await {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}
