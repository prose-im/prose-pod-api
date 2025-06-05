// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::{prelude::*, schema::*};

use super::m20240220_171150_create_member::Member;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Member::Table)
                    .add_column(string_null(NewFields::EmailAddress))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Member::Table)
                    .drop_column(NewFields::EmailAddress)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum NewFields {
    EmailAddress,
}
