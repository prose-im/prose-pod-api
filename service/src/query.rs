// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::{MemberRole, JID};
use ::entity::{member, prelude::*, server_config, workspace, workspace_invitation};
use chrono::{DateTime, Utc};
use sea_orm::*;
use uuid::Uuid;

pub struct Query;

impl Query {
    pub async fn server_config(db: &DbConn) -> Result<Option<server_config::Model>, DbErr> {
        ServerConfig::find()
            .order_by_asc(server_config::Column::Id)
            .one(db)
            .await
    }

    pub async fn workspace(db: &DbConn) -> Result<Option<workspace::Model>, DbErr> {
        Workspace::find()
            .order_by_asc(workspace::Column::Id)
            .one(db)
            .await
    }

    pub async fn get_member(db: &DbConn, jid: &JID) -> Result<Option<member::Model>, DbErr> {
        Member::find_by_jid(jid).one(db).await
    }

    pub async fn is_admin(db: &DbConn, jid: &JID) -> Result<bool, DbErr> {
        // TODO: Use a [Custom Struct](https://www.sea-ql.org/SeaORM/docs/advanced-query/custom-select/#custom-struct) to query only the `role` field.
        let member = Member::find_by_jid(jid).one(db).await?;

        // If the member is not found, do not send an error but rather send `false` as it is not an admin anyway.
        let Some(member) = member else {
            return Ok(false);
        };

        Ok(member.role == MemberRole::Admin)
    }

    pub async fn get_workspace_invitations(
        db: &DbConn,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<workspace_invitation::Model>), DbErr> {
        assert_ne!(
            page_number, 0,
            "`page_number` starts at 1 like in the public API."
        );

        let mut query =
            WorkspaceInvitation::find().order_by_asc(workspace_invitation::Column::CreatedAt);
        if let Some(until) = until {
            query = query.filter(workspace_invitation::Column::CreatedAt.lte(until));
        }
        let pages = query.paginate(db, page_size);

        let num_items_and_pages = pages.num_items_and_pages().await?;
        let models = pages.fetch_page(page_number - 1).await?;
        Ok((num_items_and_pages, models))
    }

    pub async fn get_workspace_invitation_by_id(
        db: &DbConn,
        id: &i32,
    ) -> Result<Option<workspace_invitation::Model>, DbErr> {
        workspace_invitation::Entity::find_by_id(*id).one(db).await
    }

    pub async fn get_workspace_invitation_by_accept_token(
        db: &DbConn,
        token: &Uuid,
    ) -> Result<Option<workspace_invitation::Model>, DbErr> {
        workspace_invitation::Entity::find()
            .filter(workspace_invitation::Column::AcceptToken.eq(*token))
            .one(db)
            .await
    }

    pub async fn get_workspace_invitation_by_reject_token(
        db: &DbConn,
        token: &Uuid,
    ) -> Result<Option<workspace_invitation::Model>, DbErr> {
        workspace_invitation::Entity::find()
            .filter(workspace_invitation::Column::RejectToken.eq(*token))
            .one(db)
            .await
    }

    pub async fn get_members(
        db: &DbConn,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<member::Model>), DbErr> {
        assert_ne!(
            page_number, 0,
            "`page_number` starts at 1 like in the public API."
        );

        let mut query = Member::find().order_by_asc(member::Column::JoinedAt);
        if let Some(until) = until {
            query = query.filter(member::Column::JoinedAt.lte(until));
        }
        let pages = query.paginate(db, page_size);

        let num_items_and_pages = pages.num_items_and_pages().await?;
        let models = pages.fetch_page(page_number - 1).await?;
        Ok((num_items_and_pages, models))
    }

    pub async fn get_member_count(db: &DbConn) -> Result<u64, DbErr> {
        member::Entity::find().count(db).await
    }
}
