// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, TimeDelta, Utc};
use entity::workspace_invitation::{ActiveModel, Column, Entity};
use prose_xmpp::BareJid;
use sea_orm::{
    prelude::*, DeleteResult, IntoActiveModel as _, ItemsAndPagesNumber, NotSet, QueryOrder as _,
    Set,
};
use uuid::Uuid;

use crate::{dependencies, MutationError};

use crate::model::{EmailAddress, Invitation, InvitationContact, InvitationStatus, MemberRole};

const DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME: TimeDelta = TimeDelta::days(3);

pub enum InvitationRepository {}

impl InvitationRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<InvitationCreateForm>,
        uuid: &dependencies::Uuid,
    ) -> Result<Invitation, DbErr> {
        form.into().into_active_model(uuid).insert(db).await
    }

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

    pub async fn get_by_id(
        db: &impl ConnectionTrait,
        id: &i32,
    ) -> Result<Option<Invitation>, DbErr> {
        Entity::find_by_id(*id).one(db).await
    }

    pub async fn get_by_accept_token(
        db: &impl ConnectionTrait,
        token: &Uuid,
    ) -> Result<Option<Invitation>, DbErr> {
        Entity::find()
            .filter(Column::AcceptToken.eq(*token))
            .one(db)
            .await
    }

    pub async fn get_by_reject_token(
        db: &impl ConnectionTrait,
        token: &Uuid,
    ) -> Result<Option<Invitation>, DbErr> {
        Entity::find()
            .filter(Column::RejectToken.eq(*token))
            .one(db)
            .await
    }

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

    pub async fn resend(
        db: &impl ConnectionTrait,
        uuid: &dependencies::Uuid,
        model: Invitation,
    ) -> Result<Invitation, MutationError> {
        let mut active = model.into_active_model();
        active.accept_token = Set(uuid.new_v4());
        active.accept_token_expires_at = Set(Utc::now()
            .checked_add_signed(DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME)
            .unwrap());
        let model = active.update(db).await?;

        Ok(model)
    }

    /// Accept a user invitation (i.e. delete it from database).
    /// To also create the associated user at the same time, use `UserFactory`.
    pub async fn accept(
        db: &impl ConnectionTrait,
        invitation: Invitation,
    ) -> Result<(), MutationError> {
        invitation.delete(db).await?;
        Ok(())
    }

    pub async fn count_for_email_address(
        db: &impl ConnectionTrait,
        email_address: EmailAddress,
    ) -> Result<u64, DbErr> {
        Entity::find()
            .filter(Column::EmailAddress.eq(email_address))
            .count(db)
            .await
    }

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
            accept_token_expires_at: Set(created_at
                .checked_add_signed(DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME)
                .unwrap()),
            reject_token: Set(uuid.new_v4()),
        };
        res.set_contact(self.contact);
        res
    }
}
