// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::{prelude::*, schema::*};

use super::m20240830_080808_create_pod_config::PodConfig;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // NOTE: Sqlite doesn't support multiple alter options.
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .add_column(string_null(NewFields::DashboardIpv4))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .add_column(string_null(NewFields::DashboardIpv6))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .add_column(string_null(NewFields::DashboardHostname))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // NOTE: Sqlite doesn't support multiple alter options.
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .drop_column(NewFields::DashboardHostname)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .drop_column(NewFields::DashboardIpv6)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .drop_column(NewFields::DashboardIpv4)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum NewFields {
    DashboardIpv4,
    DashboardIpv6,
    DashboardHostname,
}
