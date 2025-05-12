// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
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
                    .table(KvStore::Table)
                    .if_not_exists()
                    .primary_key(Index::create().col(KvStore::Namespace).col(KvStore::Key))
                    .col(string(KvStore::Namespace))
                    .col(string(KvStore::Key))
                    .col(json(KvStore::Value))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KvStore::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum KvStore {
    #[sea_orm(iden = "kv_store")]
    Table,
    Namespace,
    Key,
    Value,
}
