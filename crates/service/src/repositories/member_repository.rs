// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::entity::member::{ActiveModel, Column, Entity};
use chrono::{DateTime, Utc};
use prose_xmpp::BareJid;
use sea_orm::{prelude::*, ItemsAndPagesNumber, NotSet, QueryOrder as _, Set};

use crate::model::{Member, MemberRole};

pub enum MemberRepository {}

impl MemberRepository {
    /// Create the user in database but NOT on the XMPP server.
    /// Use `UserFactory` instead, to create users in both places at the same time.
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<MemberCreateForm>,
    ) -> Result<Member, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait, jid: &BareJid) -> Result<Option<Member>, DbErr> {
        Entity::find_by_jid(&jid.to_owned().into()).one(db).await
    }

    pub async fn get_all(
        db: &impl ConnectionTrait,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Member>), DbErr> {
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

    pub async fn count(db: &impl ConnectionTrait) -> Result<u64, DbErr> {
        Entity::find().count(db).await
    }

    pub async fn is_admin(db: &impl ConnectionTrait, jid: &BareJid) -> Result<bool, DbErr> {
        // TODO: Use a [Custom Struct](https://www.sea-ql.org/SeaORM/docs/advanced-query/custom-select/#custom-struct) to query only the `role` field.
        let member = Entity::find_by_jid(&jid.to_owned().into()).one(db).await?;

        // If the member is not found, do not send an error but rather send `false` as it is not an admin anyway.
        let Some(member) = member else {
            return Ok(false);
        };

        Ok(member.role == MemberRole::Admin)
    }
}

#[derive(Debug, Clone)]
pub struct MemberCreateForm {
    pub jid: BareJid,
    pub role: Option<MemberRole>,
    pub joined_at: Option<DateTime<Utc>>,
}

impl MemberCreateForm {
    fn into_active_model(self) -> ActiveModel {
        let mut res = ActiveModel {
            role: self.role.map(Set).unwrap_or(NotSet),
            joined_at: Set(self.joined_at.unwrap_or_else(Utc::now)),
            ..Default::default()
        };
        res.set_jid(&self.jid);
        res
    }
}
