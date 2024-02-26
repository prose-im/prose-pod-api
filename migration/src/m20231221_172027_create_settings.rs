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
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Settings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Settings::MessageArchiveEnabled)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_MESSAGE_ARCHIVE_ENABLED),
                    )
                    .col(
                        ColumnDef::new(Settings::MessageArchiveRetention)
                            .string()
                            .not_null()
                            .default(DEFAULT_MESSAGE_ARCHIVE_RETENTION),
                    )
                    .col(
                        ColumnDef::new(Settings::FileUploadAllowed)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_FILE_UPLOAD_ENABLED),
                    )
                    .col(
                        ColumnDef::new(Settings::FileStorageEncryptionScheme)
                            .string()
                            .not_null()
                            .default(DEFAULT_FILE_ENCRYPTION_SCHEME),
                    )
                    .col(
                        ColumnDef::new(Settings::FileStorageRetention)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(Settings::WorkspaceName).string().not_null())
                    .col(ColumnDef::new(Settings::WorkspaceIconUrl).string().null())
                    .col(ColumnDef::new(Settings::WorkspaceVCardUrl).string().null())
                    .col(
                        ColumnDef::new(Settings::WorkspaceAccentColor)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Settings::MFARequired)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_MFA_ENABLED),
                    )
                    .col(
                        ColumnDef::new(Settings::MinimumTLSVersion)
                            .string()
                            .not_null()
                            .default(DEFAULT_MINIMUM_TLS_VERSION),
                    )
                    .col(
                        ColumnDef::new(Settings::MinimumCipherSuite)
                            .string()
                            .not_null()
                            .default(DEFAULT_MINIMUM_CIPHER_SUITE),
                    )
                    .col(
                        ColumnDef::new(Settings::FederationEnabled)
                            .boolean()
                            .not_null()
                            .default(DEFAULT_FEDERATION_ENABLED),
                    )
                    .col(
                        ColumnDef::new(Settings::SettingsBackupInterval)
                            .string()
                            .not_null()
                            .default(DEFAULT_SETTINGS_BACKUP_INTERVAL),
                    )
                    .col(
                        ColumnDef::new(Settings::UserDataBackupInterval)
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
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Settings {
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
