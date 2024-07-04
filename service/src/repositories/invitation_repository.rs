// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, TimeDelta, Utc};
use entity::{
    model::{EmailAddress, InvitationContact, InvitationStatus, MemberRole},
    workspace_invitation::{ActiveModel, Column, Entity, Model},
};
use prose_xmpp::BareJid;
use sea_orm::{
    prelude::*, IntoActiveModel as _, ItemsAndPagesNumber, NotSet, QueryOrder as _, Set,
};
use uuid::Uuid;

use crate::{dependencies, MutationError};

const DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME: TimeDelta = TimeDelta::days(3);

pub enum InvitationRepository {}

impl InvitationRepository {
    pub async fn create(
        db: &DbConn,
        uuid: &dependencies::Uuid,
        jid: &BareJid,
        pre_assigned_role: MemberRole,
        contact: InvitationContact,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let mut model = ActiveModel {
            id: NotSet,
            created_at: Set(now),
            status: NotSet,
            jid: Set(jid.to_owned().into()),
            pre_assigned_role: Set(pre_assigned_role),
            invitation_channel: NotSet,
            email_address: NotSet,
            accept_token: Set(uuid.new_v4()),
            accept_token_expires_at: Set(now
                .checked_add_signed(DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME)
                .unwrap()),
            reject_token: Set(uuid.new_v4()),
        };
        model.set_contact(contact);
        model.insert(db).await
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

        let mut query = Entity::find().order_by_asc(Column::CreatedAt);
        if let Some(until) = until {
            query = query.filter(Column::CreatedAt.lte(until));
        }
        let pages = query.paginate(db, page_size);

        let num_items_and_pages = pages.num_items_and_pages().await?;
        let models = pages.fetch_page(page_number - 1).await?;
        Ok((num_items_and_pages, models))
    }

    pub async fn get_by_id(db: &DbConn, id: &i32) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(*id).one(db).await
    }

    pub async fn get_by_accept_token(db: &DbConn, token: &Uuid) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::AcceptToken.eq(*token))
            .one(db)
            .await
    }

    pub async fn get_by_reject_token(db: &DbConn, token: &Uuid) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::RejectToken.eq(*token))
            .one(db)
            .await
    }

    pub async fn update_status_by_id(
        db: &DbConn,
        id: i32,
        status: InvitationStatus,
    ) -> Result<Model, MutationError> {
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
        db: &DbConn,
        email_address: EmailAddress,
        status: InvitationStatus,
    ) -> Result<Model, MutationError> {
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
        db: &DbConn,
        model: Model,
        status: InvitationStatus,
    ) -> Result<Model, MutationError> {
        let mut active = model.into_active_model();
        active.status = Set(status);
        let model = active.update(db).await?;

        Ok(model)
    }

    pub async fn resend(
        db: &DbConn,
        uuid: &dependencies::Uuid,
        model: Model,
    ) -> Result<Model, MutationError> {
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
    pub async fn accept<'a, C: ConnectionTrait>(
        db: &C,
        invitation: Model,
    ) -> Result<(), MutationError> {
        invitation.delete(db).await?;
        Ok(())
    }
}
