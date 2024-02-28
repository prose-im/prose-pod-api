// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::model::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "server_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub message_archive_enabled: bool,
    pub message_archive_retention: PossiblyInfinite<Duration<DateLike>>,
    pub file_upload_allowed: bool,
    pub file_storage_encryption_scheme: String,
    pub file_storage_retention: PossiblyInfinite<Duration<DateLike>>,
    pub workspace_name: String,
    pub workspace_icon_url: Option<String>,
    pub workspace_v_card_url: Option<String>,
    pub workspace_accent_color: Option<String>,
    pub mfa_required: bool,
    pub minimum_tls_version: String,
    pub minimum_cipher_suite: String,
    pub federation_enabled: bool,
    pub settings_backup_interval: String,
    pub user_data_backup_interval: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
