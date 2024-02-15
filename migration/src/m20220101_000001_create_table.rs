use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

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
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Settings::MessageArchiveRetention)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Settings::FileUploadAllowed)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    // TODO: Make `FileStorageEncryptionScheme` an enum
                    .col(
                        ColumnDef::new(Settings::FileStorageEncryptionScheme)
                            .string()
                            .not_null()
                            .default("AES-256"),
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
                            .default(true),
                    )
                    // TODO: Make `MinimumTLSVersion` an enum
                    .col(
                        ColumnDef::new(Settings::MinimumTLSVersion)
                            .string()
                            .not_null()
                            .default("1.2"),
                    )
                    // TODO: Make `MinimumCipherSuite` an enum
                    .col(
                        ColumnDef::new(Settings::MinimumCipherSuite)
                            .string()
                            .not_null()
                            .default("HIGH_STRENGTH"),
                    )
                    .col(
                        ColumnDef::new(Settings::FederationEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Settings::SettingsBackupInterval)
                            .string()
                            .not_null()
                            .default("P1D"),
                    )
                    .col(
                        ColumnDef::new(Settings::UserDataBackupInterval)
                            .string()
                            .not_null()
                            .default("P1W"),
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
