// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::{prelude::*, schema::*};

use super::{
    m20240830_080808_create_pod_config::PodConfig, m20250326_095800_add_dashboard_address,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // NOTE: No one has use the API since last migration because no new version
        //   was released so we won’t bother migrating the data and just drop the columns.
        m20250326_095800_add_dashboard_address::Migration
            .down(manager)
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .add_column(string_null(NewFields::DashboardUrl))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .drop_column(NewFields::DashboardUrl)
                    .to_owned(),
            )
            .await?;
        m20250326_095800_add_dashboard_address::Migration
            .up(manager)
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum NewFields {
    DashboardUrl,
}
