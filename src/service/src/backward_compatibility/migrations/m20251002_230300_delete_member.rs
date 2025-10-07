// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::prelude::*;

use super::m20240220_171150_create_member::Member;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Member::Table).to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        super::m20240220_171150_create_member::Migration
            .up(manager)
            .await?;
        super::m20250531_231100_add_email_address::Migration
            .up(manager)
            .await?;
        Ok(())
    }
}
