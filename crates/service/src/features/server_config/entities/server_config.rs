// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::entity::prelude::*;

use crate::{
    features::{
        app_config::{AppConfig, ConfigServerDefaults},
        server_config::ServerConfig,
    },
    models::*,
};

/// XMPP server configuration, as stored in the database.
///
/// All fields are optional because the Prose Pod API only stores manual overrides.
/// This way, if security defaults are raised, every Prose Pod will automatically benefit from it upon update.
/// Those default values (from [config::defaults][crate::config::defaults]) can also be overridden
/// by a Prose Pod administrator via the Prose Pod API configuration file (`Prose.toml`).
///
/// When returning the server configuration, the Prose Pod API replaces non-overridden (empty) values
/// by their default. See [ServerConfig] and [Model::with_default_values].
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "server_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    pub domain: JidDomain,
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
    pub fn with_default_values(&self, defaults: &ConfigServerDefaults) -> ServerConfig {
        ServerConfig {
            domain: self.domain.to_owned(),
            message_archive_enabled: self.message_archive_enabled(defaults),
            message_archive_retention: self.message_archive_retention(defaults),
            file_upload_allowed: self.file_upload_allowed(defaults),
            file_storage_encryption_scheme: self
                .file_storage_encryption_scheme(defaults)
                .to_owned(),
            file_storage_retention: self.file_storage_retention(defaults),
            mfa_required: self.mfa_required(defaults),
            minimum_tls_version: self.minimum_tls_version(defaults).to_owned(),
            minimum_cipher_suite: self.minimum_cipher_suite(defaults).to_owned(),
            federation_enabled: self.federation_enabled(defaults),
            settings_backup_interval: self.settings_backup_interval(defaults).to_owned(),
            user_data_backup_interval: self.user_data_backup_interval(defaults).to_owned(),
        }
    }
    /// Same as [Model::with_default_values], used in places where we have easier access to a full [AppConfig].
    pub fn with_default_values_from(&self, app_config: &AppConfig) -> ServerConfig {
        self.with_default_values(&app_config.server.defaults)
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
