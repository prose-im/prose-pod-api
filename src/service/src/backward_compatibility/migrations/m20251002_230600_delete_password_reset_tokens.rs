// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::prelude::*;

use crate::global_storage::migrations::m20250512_131300_create_kv_store::KvStore;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let delete_stmt = Query::delete()
            .from_table(KvStore::Table)
            .cond_where(Expr::col(KvStore::Namespace).eq("auth::password_reset_tokens"))
            .to_owned();
        manager.exec_stmt(delete_stmt).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
