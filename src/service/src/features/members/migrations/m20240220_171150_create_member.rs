// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_MEMBER_ROLE: &'static str = "MEMBER";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Member::Table)
                    .if_not_exists()
                    .col(string(Member::Id).primary_key())
                    .col(string_len(Member::Role, 6).default(DEFAULT_MEMBER_ROLE))
                    .col(timestamp_with_time_zone(Member::JoinedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Member::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Member {
    Table,
    Id,
    Role,
    JoinedAt,
}
