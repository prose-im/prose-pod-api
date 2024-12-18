// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServerConfig::Table)
                    .if_not_exists()
                    .col(pk_auto(ServerConfig::Id))
                    .col(string(ServerConfig::Domain))
                    .col(boolean_null(ServerConfig::MessageArchiveEnabled))
                    .col(string_null(ServerConfig::MessageArchiveRetention))
                    .col(boolean_null(ServerConfig::FileUploadAllowed))
                    .col(string_null(ServerConfig::FileStorageEncryptionScheme))
                    .col(string_null(ServerConfig::FileStorageRetention))
                    .col(boolean_null(ServerConfig::MFARequired))
                    .col(string_null(ServerConfig::MinimumTLSVersion))
                    .col(string_null(ServerConfig::MinimumCipherSuite))
                    .col(boolean_null(ServerConfig::FederationEnabled))
                    .col(string_null(ServerConfig::SettingsBackupInterval))
                    .col(string_null(ServerConfig::UserDataBackupInterval))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServerConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(super) enum ServerConfig {
    Table,
    Id,
    Domain,
    MessageArchiveEnabled,
    MessageArchiveRetention,
    FileUploadAllowed,
    FileStorageEncryptionScheme,
    FileStorageRetention,
    MFARequired,
    MinimumTLSVersion,
    MinimumCipherSuite,
    FederationEnabled,
    SettingsBackupInterval,
    UserDataBackupInterval,
}
