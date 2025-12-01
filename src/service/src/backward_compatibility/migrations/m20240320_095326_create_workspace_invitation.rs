// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm_migration::{prelude::*, schema::*};

pub const DEFAULT_WORKSPACE_INVITATION_STATUS: &'static str = "TO_SEND";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WorkspaceInvitation::Table)
                    .if_not_exists()
                    .col(pk_auto(WorkspaceInvitation::Id))
                    .col(timestamp_with_time_zone(WorkspaceInvitation::CreatedAt))
                    .col(string(WorkspaceInvitation::Jid))
                    .col(
                        string(WorkspaceInvitation::Status)
                            .default(DEFAULT_WORKSPACE_INVITATION_STATUS),
                    )
                    .col(string(WorkspaceInvitation::PreAssignedRole))
                    .col(string(WorkspaceInvitation::InvitationChannel))
                    .col(string_null(WorkspaceInvitation::EmailAddress))
                    .col(string(WorkspaceInvitation::AcceptToken))
                    .col(string(WorkspaceInvitation::AcceptTokenExpiresAt))
                    .col(string(WorkspaceInvitation::RejectToken))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WorkspaceInvitation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(super) enum WorkspaceInvitation {
    Table,
    Id,
    CreatedAt,
    Jid,
    Status,
    PreAssignedRole,
    InvitationChannel,
    EmailAddress,
    AcceptToken,
    AcceptTokenExpiresAt,
    RejectToken,
}
