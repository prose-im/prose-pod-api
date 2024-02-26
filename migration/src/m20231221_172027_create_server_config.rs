use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Enable message archiving by default.
pub const DEFAULT_MESSAGE_ARCHIVE_ENABLED: bool = true;
/// 2 years in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601#Durations).
pub const DEFAULT_MESSAGE_ARCHIVE_RETENTION: &'static str = "P2Y";
/// Enable file upload by default.
pub const DEFAULT_FILE_UPLOAD_ENABLED: bool = true;
// TODO: Make `FileStorageEncryptionScheme` an enum
/// Encrypt files in [AES 256](https://fr.wikipedia.org/wiki/Advanced_Encryption_Standard) by default.
pub const DEFAULT_FILE_ENCRYPTION_SCHEME: &'static str = "AES-256";
/// Enable MFA by default.
pub const DEFAULT_MFA_ENABLED: bool = true;
// TODO: Make `MinimumTLSVersion` an enum
/// Default minimum [TLS](https://fr.wikipedia.org/wiki/Transport_Layer_Security) version.
pub const DEFAULT_MINIMUM_TLS_VERSION: &'static str = "1.2";
// TODO: Make `MinimumCipherSuite` an enum
/// High security by default.
pub const DEFAULT_MINIMUM_CIPHER_SUITE: &'static str = "HIGH_STRENGTH";
/// Enable federation by default.
pub const DEFAULT_FEDERATION_ENABLED: bool = true;
/// 1 day in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601#Durations).
pub const DEFAULT_SETTINGS_BACKUP_INTERVAL: &'static str = "P1D";
/// 1 week in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601#Durations).
pub const DEFAULT_USER_DATA_BACKUP_INTERVAL: &'static str = "P1W";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServerConfig::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServerConfig::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::MessageArchiveEnabled)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_MESSAGE_ARCHIVE_ENABLED),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::MessageArchiveRetention)
                            .string()
                            .not_null()
                            .default(DEFAULT_MESSAGE_ARCHIVE_RETENTION),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::FileUploadAllowed)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_FILE_UPLOAD_ENABLED),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::FileStorageEncryptionScheme)
                            .string()
                            .not_null()
                            .default(DEFAULT_FILE_ENCRYPTION_SCHEME),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::FileStorageRetention)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::WorkspaceName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::WorkspaceIconUrl)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::WorkspaceVCardUrl)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::WorkspaceAccentColor)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::MFARequired)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_MFA_ENABLED),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::MinimumTLSVersion)
                            .string()
                            .not_null()
                            .default(DEFAULT_MINIMUM_TLS_VERSION),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::MinimumCipherSuite)
                            .string()
                            .not_null()
                            .default(DEFAULT_MINIMUM_CIPHER_SUITE),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::FederationEnabled)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_FEDERATION_ENABLED),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::SettingsBackupInterval)
                            .string()
                            .not_null()
                            .default(DEFAULT_SETTINGS_BACKUP_INTERVAL),
                    )
                    .col(
                        ColumnDef::new(ServerConfig::UserDataBackupInterval)
                            .string()
                            .not_null()
                            .default(DEFAULT_USER_DATA_BACKUP_INTERVAL),
                    )
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
enum ServerConfig {
    Table,
    Id,
    MessageArchiveEnabled,
    MessageArchiveRetention,
    FileUploadAllowed,
    FileStorageEncryptionScheme,
    FileStorageRetention,
    WorkspaceName,
    WorkspaceIconUrl,
    WorkspaceVCardUrl,
    WorkspaceAccentColor,
    MFARequired,
    MinimumTLSVersion,
    MinimumCipherSuite,
    FederationEnabled,
    SettingsBackupInterval,
    UserDataBackupInterval,
}
