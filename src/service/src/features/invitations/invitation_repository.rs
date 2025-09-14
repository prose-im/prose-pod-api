// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, TimeDelta, Utc};
use sea_orm::{
    prelude::*, DeleteResult, IntoActiveModel as _, ItemsAndPagesNumber, NotSet, QueryOrder as _,
    Set,
};
use secrecy::{zeroize::Zeroize, ExposeSecret, SecretString, SerializableSecret};
use serdev::{Deserialize, Serialize};

use crate::{
    dependencies,
    invitations::{
        entities::workspace_invitation::{ActiveModel, Column, Entity},
        Invitation, InvitationContact, InvitationStatus,
    },
    members::MemberRole,
    models::{BareJid, EmailAddress},
    MutationError,
};

#[derive(Debug)]
pub enum InvitationRepository {}

impl InvitationRepository {
    // TODO: Trace fields.
    #[tracing::instrument(name = "db::invitation::create", level = "info", skip_all, err)]
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<InvitationCreateForm>,
        uuid: &dependencies::Uuid,
    ) -> Result<Invitation, DbErr> {
        form.into().into_active_model(uuid).insert(db).await
    }

    #[tracing::instrument(
        name = "db::invitation::get_all",
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
    ) -> Result<(ItemsAndPagesNumber, Vec<Invitation>), DbErr> {
        assert_ne!(
            page_number, 0,
            "`page_number` starts at 1 like in the public API."
        );

        let mut query = Entity::find().order_by_asc(Column::CreatedAt);
        if let Some(until) = until {
            query = query.filter(Column::CreatedAt.lte(until));
        }
        let pages = query.paginate(db, page_size);

        let num_items_and_pages = pages.num_items_and_pages().await?;
        let models = pages.fetch_page(page_number - 1).await?;
        Ok((num_items_and_pages, models))
    }

    #[tracing::instrument(
        name = "db::invitation::get_by_id",
        level = "trace",
        skip_all,
        fields(invitation_id = id),
        err
    )]
    pub async fn get_by_id(
        db: &impl ConnectionTrait,
        id: &i32,
    ) -> Result<Option<Invitation>, DbErr> {
        Entity::find_by_id(*id).one(db).await
    }

    #[tracing::instrument(
        name = "db::invitation::get_by_jid", level = "trace",
        skip_all, fields(jid = jid.to_string()),
        err
    )]
    pub async fn get_by_jid(
        db: &impl ConnectionTrait,
        jid: &BareJid,
    ) -> Result<Option<Invitation>, DbErr> {
        Entity::find()
            .filter(Column::Jid.eq(jid.as_str()))
            .one(db)
            .await
    }

    #[tracing::instrument(
        name = "db::invitation::get_by_accept_token",
        level = "trace",
        skip_all,
        err
    )]
    pub async fn get_by_accept_token(
        db: &impl ConnectionTrait,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        Entity::find()
            .filter(Column::AcceptToken.eq(*token.expose_secret()))
            .one(db)
            .await
    }

    #[tracing::instrument(
        name = "db::invitation::get_by_reject_token",
        level = "trace",
        skip_all,
        err
    )]
    pub async fn get_by_reject_token(
        db: &impl ConnectionTrait,
        token: InvitationToken,
    ) -> Result<Option<Invitation>, DbErr> {
        Entity::find()
            .filter(Column::RejectToken.eq(*token.expose_secret()))
            .one(db)
            .await
    }

    #[tracing::instrument(
        name = "db::invitation::update_status_by_id",
        level = "trace",
        skip_all,
        fields(invitation_id = id, status),
        err
    )]
    pub async fn update_status_by_id(
        db: &impl ConnectionTrait,
        id: i32,
        status: InvitationStatus,
    ) -> Result<Invitation, MutationError> {
        // Query
        let model = Entity::find_by_id(id).one(db).await?;
        let Some(model) = model else {
            return Err(MutationError::EntityNotFound {
                entity_name: stringify!(workspace_invitation::Entity),
            });
        };

        // Update
        Self::update_status(db, model, status).await
    }

    #[tracing::instrument(
        name = "db::invitation::update_status_by_email",
        level = "trace",
        skip_all,
        fields(email_address, status),
        err
    )]
    pub async fn update_status_by_email(
        db: &impl ConnectionTrait,
        email_address: EmailAddress,
        status: InvitationStatus,
    ) -> Result<Invitation, MutationError> {
        // Query
        let model = Entity::find()
            .filter(Column::EmailAddress.eq(email_address))
            .one(db)
            .await?;
        let Some(model) = model else {
            return Err(MutationError::EntityNotFound {
                entity_name: stringify!(workspace_invitation::Entity),
            });
        };

        // Update
        Self::update_status(db, model, status).await
    }

    #[tracing::instrument(
        name = "db::invitation::update_status",
        level = "info",
        skip_all,
        fields(invitation_id = model.id, jid = model.jid.to_string(), previous_status = model.status.to_string(), status = status.to_string()),
        err
    )]
    pub async fn update_status(
        db: &impl ConnectionTrait,
        model: Invitation,
        status: InvitationStatus,
    ) -> Result<Invitation, MutationError> {
        let mut active = model.into_active_model();
        active.status = Set(status);
        let model = active.update(db).await?;

        Ok(model)
    }

    #[tracing::instrument(
        name = "db::invitation::resend",
        level = "info",
        skip_all,
        fields(invitation_id = model.id, jid = model.jid.to_string(), status = model.status.to_string(), ttl),
        err
    )]
    pub async fn resend(
        db: &impl ConnectionTrait,
        uuid: &dependencies::Uuid,
        model: Invitation,
        ttl: TimeDelta,
    ) -> Result<Invitation, MutationError> {
        let mut active = model.into_active_model();
        active.accept_token = Set(uuid.new_v4());
        active.accept_token_expires_at = Set(Utc::now().checked_add_signed(ttl).unwrap());
        let model = active.update(db).await?;

        Ok(model)
    }

    /// Accept a user invitation (i.e. delete it from database).
    /// To also create the associated user at the same time,
    /// use [`MemberService`][crate::members::MemberService].
    #[tracing::instrument(
        name = "db::invitation::accept",
        level = "info",
        skip_all,
        fields(invitation_id = invitation.id, jid = invitation.jid.to_string()),
        err
    )]
    pub async fn accept(
        db: &impl ConnectionTrait,
        invitation: Invitation,
    ) -> Result<(), MutationError> {
        invitation.delete(db).await?;
        Ok(())
    }

    #[tracing::instrument(
        name = "db::invitation::count_for_email_address",
        level = "trace",
        skip_all,
        fields(email_address),
        err
    )]
    pub async fn count_for_email_address(
        db: &impl ConnectionTrait,
        email_address: EmailAddress,
    ) -> Result<u64, DbErr> {
        Entity::find()
            .filter(Column::EmailAddress.eq(email_address))
            .count(db)
            .await
    }

    #[tracing::instrument(
        name = "db::invitation::delete_by_id",
        level = "info",
        skip_all,
        fields(invitation_id = invitation_id),
        err
    )]
    pub async fn delete_by_id(
        db: &impl ConnectionTrait,
        invitation_id: i32,
    ) -> Result<DeleteResult, DbErr> {
        Entity::delete_by_id(invitation_id).exec(db).await
    }
}

#[derive(Debug, Clone)]
pub struct InvitationCreateForm {
    pub jid: BareJid,
    pub pre_assigned_role: Option<MemberRole>,
    pub contact: InvitationContact,
    pub created_at: Option<DateTime<Utc>>,
    pub ttl: TimeDelta,
}

impl InvitationCreateForm {
    fn into_active_model(self, uuid: &dependencies::Uuid) -> ActiveModel {
        let created_at = self.created_at.unwrap_or_else(Utc::now);
        let mut res = ActiveModel {
            id: NotSet,
            created_at: Set(created_at),
            status: NotSet,
            jid: Set(self.jid.to_owned().into()),
            pre_assigned_role: Set(self.pre_assigned_role.unwrap_or_default()),
            invitation_channel: NotSet,
            email_address: NotSet,
            accept_token: Set(uuid.new_v4()),
            accept_token_expires_at: Set(created_at.checked_add_signed(self.ttl).unwrap()),
            reject_token: Set(uuid.new_v4()),
        };
        res.set_contact(self.contact);
        res
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
// NOTE: No need to validate as `Uuid` is parsed.
#[repr(transparent)]
pub struct InvitationToken(Uuid);
impl Zeroize for InvitationToken {
    fn zeroize(&mut self) {
        self.0.into_bytes().zeroize()
    }
}
impl SerializableSecret for InvitationToken {}
impl ExposeSecret<Uuid> for InvitationToken {
    fn expose_secret(&self) -> &Uuid {
        &self.0
    }
}
impl From<Uuid> for InvitationToken {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
impl InvitationToken {
    pub fn into_secret_string(&self) -> SecretString {
        SecretString::from(self.0.to_string())
    }
}
impl Into<SecretString> for InvitationToken {
    fn into(self) -> SecretString {
        self.into_secret_string()
    }
}
