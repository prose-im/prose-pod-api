// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use sea_orm::{
    prelude::*, DeleteResult, ItemsAndPagesNumber, NotSet, QueryOrder as _, QuerySelect, Set,
};
use tracing::instrument;

use crate::{
    members::{
        entities::member::{ActiveModel, Column, Entity, Model},
        MemberRole,
    },
    models::{BareJid, EmailAddress},
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
    ) -> Result<Model, DbErr> {
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
        name = "db::member::exists", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    pub async fn exists(db: &impl ConnectionTrait, jid: &BareJid) -> Result<bool, DbErr> {
        match Entity::find_by_jid(&jid.to_owned().into()).count(db).await {
            Ok(count) => Ok(count > 0),
            Err(err) => Err(err),
        }
    }

    #[instrument(
        name = "db::member::get", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    pub async fn get(db: &impl ConnectionTrait, jid: &BareJid) -> Result<Option<Model>, DbErr> {
        Entity::find_by_jid(&jid.to_owned().into()).one(db).await
    }

    #[instrument(
        name = "db::member::get_page",
        level = "trace",
        skip_all,
        fields(page_number, page_size, until),
        err
    )]
    pub async fn get_page(
        db: &impl ConnectionTrait,
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

    #[instrument(name = "db::member::get_all", level = "trace", skip_all, err)]
    pub async fn get_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, DbErr> {
        Entity::find().order_by_asc(Column::JoinedAt).all(db).await
    }

    #[instrument(name = "db::member::get_all_until", level = "trace", skip_all, err)]
    #[inline]
    pub async fn get_all_until(
        db: &impl ConnectionTrait,
        until: Option<DateTime<Utc>>,
    ) -> Result<Vec<Model>, DbErr> {
        let mut query = Entity::find().order_by_asc(Column::JoinedAt);
        if let Some(until) = until {
            query = query.filter(Column::JoinedAt.lte(until));
        }
        query.all(db).await
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

    /// Updates a member’s role in database.
    ///
    /// Returns `None` if the role hasn’t changed.
    ///
    /// Returns the **old** value if the role has changed.
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
    ) -> Result<Option<MemberRole>, DbErr> {
        let member_role = Entity::find_by_jid(jid)
            .select_only()
            .columns([Column::Role])
            .into_tuple::<MemberRole>()
            .one(db)
            .await?;

        let Some(member_role) = member_role else {
            return Err(DbErr::RecordNotFound(format!("No member with id '{jid}'.")));
        };

        // Abort if no change needed.
        if member_role == role {
            return Ok(None);
        }

        let mut member = <ActiveModel as ActiveModelTrait>::default();
        member.set_jid(jid);
        member.role = Set(role);
        member.update(db).await?;

        Ok(Some(member_role))
    }

    /// Updates a member’s email address in database.
    ///
    /// Returns `None` if the email address hasn’t changed.
    ///
    /// Returns the **old** value if the email address has changed.
    #[instrument(
        name = "db::member::set_email_address", level = "trace",
        skip_all, fields(
            jid = jid.to_string(),
            email_address = email_address.as_ref().map(ToString::to_string)
        ),
        err
    )]
    pub async fn set_email_address(
        db: &impl ConnectionTrait,
        jid: &BareJid,
        email_address: Option<EmailAddress>,
    ) -> Result<Option<Option<EmailAddress>>, DbErr> {
        let old_email = Entity::find_by_jid(jid)
            .select_only()
            .columns([Column::EmailAddress])
            .into_tuple::<Option<EmailAddress>>()
            .one(db)
            .await?;

        let Some(old_email) = old_email else {
            return Err(DbErr::RecordNotFound(format!("No member with id '{jid}'.")));
        };

        // Abort if no change needed.
        if old_email == email_address {
            return Ok(None);
        }

        let mut member = <ActiveModel as ActiveModelTrait>::default();
        member.set_jid(jid);
        member.email_address = Set(email_address);
        member.update(db).await?;

        Ok(Some(old_email))
    }
}

#[derive(Debug, Clone)]
pub struct MemberCreateForm {
    pub jid: BareJid,
    pub role: Option<MemberRole>,
    pub joined_at: Option<DateTime<Utc>>,
    pub email_address: Option<EmailAddress>,
}

impl MemberCreateForm {
    fn into_active_model(self) -> ActiveModel {
        let mut res = ActiveModel {
            id: NotSet,
            role: self.role.map(Set).unwrap_or(NotSet),
            joined_at: Set(self.joined_at.unwrap_or_else(Utc::now)),
            email_address: Set(self.email_address),
        };
        res.set_jid(&self.jid);
        res
    }
}
