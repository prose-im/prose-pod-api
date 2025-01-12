// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m20231221_172027_create_server_config::ServerConfig;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .add_column(string_len_null(NewFields::TlsProfile, 12))
                    .to_owned(),
            )
            .await?;
        // NOTE: We should migrate data from `minimim_tls_version` to `tls_profile`
        //   but Prose isn't used in production yet so let's not bother.
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .drop_column(ServerConfig::MinimumTLSVersion)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .drop_column(ServerConfig::MinimumCipherSuite)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .add_column(string_null(ServerConfig::MinimumTLSVersion))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .add_column(string_null(ServerConfig::MinimumCipherSuite))
                    .to_owned(),
            )
            .await?;
        // NOTE: We should migrate data from `tls_profile` to `minimim_tls_version`
        //   but Prose isn't used in production yet so let's not bother.
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .drop_column(NewFields::TlsProfile)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum NewFields {
    TlsProfile,
}
