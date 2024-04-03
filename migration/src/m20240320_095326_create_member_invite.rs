use sea_orm_migration::{prelude::*, schema::*};

pub const DEFAULT_MEMBER_INVITE_STATE: &'static str = "TO_SEND";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MemberInvite::Table)
                    .if_not_exists()
                    .col(pk_auto(MemberInvite::Id))
                    .col(timestamp_with_time_zone(MemberInvite::CreatedAt))
                    .col(string(MemberInvite::Jid))
                    .col(string(MemberInvite::State).default(DEFAULT_MEMBER_INVITE_STATE))
                    .col(string(MemberInvite::PreAssignedRole))
                    .col(string(MemberInvite::InvitationChannel))
                    .col(string_null(MemberInvite::EmailAddress))
                    .col(string(MemberInvite::AcceptToken))
                    .col(string(MemberInvite::AcceptTokenExpiresAt))
                    .col(string(MemberInvite::RejectToken))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MemberInvite::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MemberInvite {
    Table,
    Id,
    CreatedAt,
    Jid,
    State,
    PreAssignedRole,
    InvitationChannel,
    EmailAddress,
    AcceptToken,
    AcceptTokenExpiresAt,
    RejectToken,
}
