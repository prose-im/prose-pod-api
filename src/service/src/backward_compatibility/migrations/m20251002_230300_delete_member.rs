// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::prelude::*;

use crate::{
    backward_compatibility::migrations::m20250531_231100_add_email_address::NewFields,
    global_storage::migrations::m20250512_131300_create_kv_store::KvStore,
    util::sea_orm::JsonQuote,
};

use super::m20240220_171150_create_member::Member;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let select = Query::select()
            .expr(Expr::val("recovery_emails")) // namespace
            .column(Member::Id) // key
            .expr(Func::cust(JsonQuote).arg(Expr::col(NewFields::EmailAddress))) // value
            .from(Member::Table)
            .and_where(Expr::col(NewFields::EmailAddress).is_not_null())
            .to_owned();

        let insert = Query::insert()
            .into_table(KvStore::Table)
            .columns([
                KvStore::Namespace,
                KvStore::Key,
                KvStore::Value,
            ])
            .on_conflict(
                OnConflict::columns([
                    KvStore::Namespace,
                    KvStore::Key,
                ])
                .do_nothing()
                .to_owned(),
            )
            .select_from(select)
            .unwrap_or_else(|err| match err {
                sea_query::error::Error::ColValNumMismatch { .. } => panic!("{err}"),
            })
            .to_owned();

        manager.exec_stmt(insert.to_owned()).await?;

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
