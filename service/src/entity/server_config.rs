// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Cow;

use jid::{DomainPart, DomainRef};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{config::ConfigServerDefaults, model::*};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "server_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing, skip_deserializing)]
    id: i32,
    pub domain: String,
    pub message_archive_enabled: Option<bool>,
    pub message_archive_retention: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub file_upload_allowed: Option<bool>,
    pub file_storage_encryption_scheme: Option<String>,
    pub file_storage_retention: Option<PossiblyInfinite<Duration<DateLike>>>,
    pub mfa_required: Option<bool>,
    pub minimum_tls_version: Option<String>,
    pub minimum_cipher_suite: Option<String>,
    pub federation_enabled: Option<bool>,
    pub settings_backup_interval: Option<String>,
    pub user_data_backup_interval: Option<String>,
}

impl Model {
    pub fn domain(&self) -> Cow<DomainRef> {
        DomainPart::new(&self.domain).unwrap_or_else(|err| panic!("Invalid domain: {err}"))
    }
}

macro_rules! get_or_default {
    ($var:ident, $t:ty) => {
        pub fn $var(&self, defaults: &ConfigServerDefaults) -> $t {
            self.$var.unwrap_or(defaults.$var)
        }
    };
}
macro_rules! get_or_default_string {
    ($var:ident) => {
        pub fn $var<'a, 'b>(&'a self, defaults: &'b ConfigServerDefaults) -> &'a str
        where
            'b: 'a,
        {
            match self.$var.as_deref() {
                Some(s) => s,
                None => &defaults.$var,
            }
        }
    };
}

impl Model {
    get_or_default!(message_archive_enabled, bool);
    get_or_default!(
        message_archive_retention,
        PossiblyInfinite<Duration<DateLike>>
    );
    get_or_default!(file_upload_allowed, bool);
    get_or_default_string!(file_storage_encryption_scheme);
    get_or_default!(file_storage_retention, PossiblyInfinite<Duration<DateLike>>);
    get_or_default!(mfa_required, bool);
    get_or_default_string!(minimum_tls_version);
    get_or_default_string!(minimum_cipher_suite);
    get_or_default!(federation_enabled, bool);
    get_or_default_string!(settings_backup_interval);
    get_or_default_string!(user_data_backup_interval);
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
