// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
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
                    .table(PodConfig::Table)
                    .if_not_exists()
                    .col(pk_auto(PodConfig::Id))
                    .col(string_null(PodConfig::Ipv4))
                    .col(string_null(PodConfig::Ipv6))
                    .col(string_null(PodConfig::Hostname))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PodConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(super) enum PodConfig {
    Table,
    Id,
    Ipv4,
    Ipv6,
    Hostname,
}
