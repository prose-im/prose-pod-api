use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Enable message archiving by default.
pub const DEFAULT_MESSAGE_ARCHIVE_ENABLED: bool = true;
/// Keep indefinitely, as defined in <https://github.com/prose-im/prose-pod-system/blob/f2e353758e628c01c0923fc0e46491f1644354c9/server/etc/prosody/prosody.cfg.lua#L97>.
pub const DEFAULT_MESSAGE_ARCHIVE_RETENTION: &'static str = "infinite";
/// Enable file upload by default.
pub const DEFAULT_FILE_UPLOAD_ENABLED: bool = true;
// TODO: Make `FileStorageEncryptionScheme` an enum
/// Encrypt files in [AES 256](https://fr.wikipedia.org/wiki/Advanced_Encryption_Standard) by default.
pub const DEFAULT_FILE_ENCRYPTION_SCHEME: &'static str = "AES-256";
/// Keep indefinitely, as defined in <https://github.com/prose-im/prose-pod-system/blob/f2e353758e628c01c0923fc0e46491f1644354c9/server/etc/prosody/prosody.cfg.lua#L126>.
pub const DEFAULT_FILE_RETENTION: &'static str = "infinite";
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
                    .col(pk_auto(ServerConfig::Id))
                    .col(
                        boolean(ServerConfig::MessageArchiveEnabled)
                            .default(DEFAULT_MESSAGE_ARCHIVE_ENABLED),
                    )
                    .col(
                        string(ServerConfig::MessageArchiveRetention)
                            .default(DEFAULT_MESSAGE_ARCHIVE_RETENTION),
                    )
                    .col(
                        boolean(ServerConfig::FileUploadAllowed)
                            .default(DEFAULT_FILE_UPLOAD_ENABLED),
                    )
                    .col(
                        string(ServerConfig::FileStorageEncryptionScheme)
                            .default(DEFAULT_FILE_ENCRYPTION_SCHEME),
                    )
                    .col(string(ServerConfig::FileStorageRetention).default(DEFAULT_FILE_RETENTION))
                    .col(string(ServerConfig::WorkspaceName))
                    .col(string_null(ServerConfig::WorkspaceIconUrl))
                    .col(string_null(ServerConfig::WorkspaceVCardUrl))
                    .col(string_null(ServerConfig::WorkspaceAccentColor))
                    .col(boolean(ServerConfig::MFARequired).default(DEFAULT_MFA_ENABLED))
                    .col(
                        string(ServerConfig::MinimumTLSVersion)
                            .default(DEFAULT_MINIMUM_TLS_VERSION),
                    )
                    .col(
                        string(ServerConfig::MinimumCipherSuite)
                            .default(DEFAULT_MINIMUM_CIPHER_SUITE),
                    )
                    .col(
                        boolean(ServerConfig::FederationEnabled)
                            .default(DEFAULT_FEDERATION_ENABLED),
                    )
                    .col(
                        string(ServerConfig::SettingsBackupInterval)
                            .default(DEFAULT_SETTINGS_BACKUP_INTERVAL),
                    )
                    .col(
                        string(ServerConfig::UserDataBackupInterval)
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
