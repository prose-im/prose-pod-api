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
                    .col(string(WorkspaceInvitation::Username))
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
enum WorkspaceInvitation {
    Table,
    Id,
    CreatedAt,
    Username,
    Status,
    PreAssignedRole,
    InvitationChannel,
    EmailAddress,
    AcceptToken,
    AcceptTokenExpiresAt,
    RejectToken,
}
