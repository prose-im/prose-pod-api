// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use entity::{
    member::{ActiveModel, Column, Entity, Model},
    model::MemberRole,
};
use prose_xmpp::BareJid;
use sea_orm::{prelude::*, ItemsAndPagesNumber, NotSet, QueryOrder as _, Set};

use crate::MutationError;

pub enum MemberRepository {}

impl MemberRepository {
    /// Create the user in database but NOT on the XMPP server.
    /// Use `UserFactory` instead, to create users in both places at the same time.
    pub async fn create<'a, C: ConnectionTrait>(
        db: &C,
        jid: &BareJid,
        role: &Option<MemberRole>,
    ) -> Result<Model, MutationError> {
        let now = Utc::now();
        let mut new_member = ActiveModel {
            id: NotSet,
            role: role.map(Set).unwrap_or(NotSet),
            joined_at: Set(now),
        };
        new_member.set_jid(&jid.to_owned().into());
        new_member.insert(db).await.map_err(Into::into)
    }

    pub async fn get(db: &DbConn, jid: &BareJid) -> Result<Option<Model>, DbErr> {
        Entity::find_by_jid(&jid.to_owned().into()).one(db).await
    }

    pub async fn get_all(
        db: &DbConn,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Model>), DbErr> {
        assert_ne!(
            page_number, 0,
            "`page_number` starts at 1 like in the public API."
        );

        let mut query = Entity::find().order_by_asc(Column::JoinedAt);
        if let Some(until) = until {
            query = query.filter(Column::JoinedAt.lte(until));
        }
        let pages = query.paginate(db, page_size);

        let num_items_and_pages = pages.num_items_and_pages().await?;
        let models = pages.fetch_page(page_number - 1).await?;
        Ok((num_items_and_pages, models))
    }

    pub async fn count(db: &DbConn) -> Result<u64, DbErr> {
        Entity::find().count(db).await
    }

    pub async fn is_admin(db: &DbConn, jid: &BareJid) -> Result<bool, DbErr> {
        // TODO: Use a [Custom Struct](https://www.sea-ql.org/SeaORM/docs/advanced-query/custom-select/#custom-struct) to query only the `role` field.
        let member = Entity::find_by_jid(&jid.to_owned().into()).one(db).await?;

        // If the member is not found, do not send an error but rather send `false` as it is not an admin anyway.
        let Some(member) = member else {
            return Ok(false);
        };

        Ok(member.role == MemberRole::Admin)
    }
}
