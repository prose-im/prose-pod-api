// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::{prelude::*, schema::*};

use super::m20240830_080808_create_pod_config::PodConfig;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .add_column(string_null(NewFields::DashboardUrl))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PodConfig::Table)
                    .drop_column(NewFields::DashboardUrl)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum NewFields {
    DashboardUrl,
}
