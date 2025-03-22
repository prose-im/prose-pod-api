// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use sea_orm::{
    prelude::*, DeleteResult, IntoActiveModel, ItemsAndPagesNumber, NotSet, QueryOrder as _, Set,
};
use tracing::instrument;

use crate::{
    members::{
        member::{ActiveModel, Column, Entity},
        Member, MemberRole,
    },
    models::BareJid,
};

#[derive(Debug, Clone)]
pub enum MemberRepository {}

impl MemberRepository {
    /// Create the user in database but NOT on the XMPP server.
    /// Use [`MemberService`][crate::members::MemberService] instead,
    /// to create users in both places at the same time.
    #[instrument(name = "db::member::create", level = "trace", skip_all, err)]
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<MemberCreateForm>,
    ) -> Result<Member, DbErr> {
        let form: MemberCreateForm = form.into();
        tracing::Span::current()
            .record("jid", form.jid.to_string())
            .record("role", form.role.map(|role| role.to_string()));
        form.into_active_model().insert(db).await
    }

    /// Delete the user from database but NOT from the XMPP server.
    /// Use [`MemberService`][crate::members::MemberService] instead,
    /// to delete users from both places at the same time.
    #[instrument(
        name = "db::member::delete", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    pub async fn delete(
        db: &impl ConnectionTrait,
        jid: &BareJid,
    ) -> Result<Option<DeleteResult>, DbErr> {
        match Self::get(db, jid).await? {
            Some(model) => Ok(Some(model.delete(db).await?)),
            None => Ok(None),
        }
    }

    #[instrument(
        name = "db::member::get", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    pub async fn get(db: &impl ConnectionTrait, jid: &BareJid) -> Result<Option<Member>, DbErr> {
        Entity::find_by_jid(&jid.to_owned().into()).one(db).await
    }

    #[instrument(
        name = "db::member::get_all",
        level = "trace",
        skip_all,
        fields(page_number, page_size, until),
        err
    )]
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

    #[instrument(name = "db::member::get_count", level = "trace", skip_all, err)]
    pub async fn count(db: &impl ConnectionTrait) -> Result<u64, DbErr> {
        Entity::find().count(db).await
    }

    #[instrument(
        name = "db::member::is_admin", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    pub async fn is_admin(db: &impl ConnectionTrait, jid: &BareJid) -> Result<bool, DbErr> {
        // TODO: Use a [Custom Struct](https://www.sea-ql.org/SeaORM/docs/advanced-query/custom-select/#custom-struct) to query only the `role` field.
        let member = Entity::find_by_jid(jid).one(db).await?;

        // If the member is not found, do not send an error but rather send `false` as it is not an admin anyway.
        let Some(member) = member else {
            return Ok(false);
        };

        Ok(member.role == MemberRole::Admin)
    }

    #[instrument(
        name = "db::member::set_role", level = "trace",
        skip_all, fields(
            jid = jid.to_string(),
            role = role.to_string()
        ),
        err
    )]
    pub async fn set_role(
        db: &impl ConnectionTrait,
        jid: &BareJid,
        role: MemberRole,
    ) -> Result<Option<Member>, DbErr> {
        // TODO: Use a [Custom Struct](https://www.sea-ql.org/SeaORM/docs/advanced-query/custom-select/#custom-struct) to query only the `role` field.
        let member = Entity::find_by_jid(jid).one(db).await?;

        let Some(member) = member else {
            return Err(DbErr::RecordNotFound(format!("No member with id '{jid}'.")));
        };

        // Abort if no change needed.
        if member.role == role {
            return Ok(None);
        }

        let mut member = member.into_active_model();
        member.role = Set(role);

        member.update(db).await.map(Option::Some)
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
