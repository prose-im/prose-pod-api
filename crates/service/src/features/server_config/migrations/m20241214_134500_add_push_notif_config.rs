// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::m20231221_172027_create_server_config::ServerConfig;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .add_column(boolean_null(NotifConfig::PushNotificationWithBody))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .add_column(boolean_null(NotifConfig::PushNotificationWithSender))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .drop_column(NotifConfig::PushNotificationWithBody)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ServerConfig::Table)
                    .drop_column(NotifConfig::PushNotificationWithSender)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum NotifConfig {
    PushNotificationWithBody,
    PushNotificationWithSender,
}
