// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;

use ::entity::{
    member,
    model::{EmailAddress, MemberRole, JID},
    server_config, workspace,
    workspace_invitation::{self, InvitationContact, InvitationStatus},
};
use chrono::{prelude::Utc, TimeDelta};
use sea_orm::{prelude::*, ActiveValue::NotSet, IntoActiveModel as _, Set};

use crate::dependencies::Uuid;

const DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME: TimeDelta = TimeDelta::days(3);

pub struct Mutation;

impl Mutation {
    pub async fn create_server_config<'a, C: ConnectionTrait>(
        db: &C,
        form_data: server_config::ActiveModel,
    ) -> Result<server_config::Model, DbErr> {
        form_data.insert(db).await
    }
    pub async fn create_workspace<'a, C: ConnectionTrait>(
        db: &C,
        form_data: workspace::ActiveModel,
    ) -> Result<workspace::Model, DbErr> {
        form_data.insert(db).await
    }

    // pub async fn update_server_config_by_id(
    //     db: &DbConn,
    //     id: i32,
    //     form_data: server_config::Model,
    // ) -> Result<server_config::Model, DbErr> {
    //     let server_config: server_config::ActiveModel = ServerConfig::find_by_id(id)
    //         .one(db)
    //         .await?
    //         .ok_or(DbErr::Custom("Cannot find server_config.".to_owned()))
    //         .map(Into::into)?;

    //     server_config::ActiveModel {
    //         id: server_config.id,
    //         title: Set(form_data.title.to_owned()),
    //         text: Set(form_data.text.to_owned()),
    //     }
    //     .update(db)
    //     .await
    // }

    pub async fn create_workspace_invitation(
        db: &DbConn,
        uuid: &Uuid,
        jid: &JID,
        pre_assigned_role: MemberRole,
        contact: InvitationContact,
    ) -> Result<workspace_invitation::Model, DbErr> {
        let mut model = workspace_invitation::ActiveModel::new();
        let now = Utc::now();
        model.created_at = Set(now);
        model.jid = Set(jid.to_owned());
        model.pre_assigned_role = Set(pre_assigned_role);
        model.set_contact(contact);
        model.accept_token = Set(uuid.new_v4());
        model.accept_token_expires_at = Set(now
            .checked_add_signed(DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME)
            .unwrap());
        model.reject_token = Set(uuid.new_v4());
        model.insert(db).await
    }

    pub async fn update_workspace_invitation_status_by_id(
        db: &DbConn,
        id: i32,
        status: InvitationStatus,
    ) -> Result<workspace_invitation::Model, MutationError> {
        // Query
        let model = workspace_invitation::Entity::find_by_id(id).one(db).await?;
        let Some(model) = model else {
            return Err(MutationError::EntityNotFound {
                entity_name: stringify!(workspace_invitation::Entity),
            });
        };

        // Update
        Self::update_workspace_invitation_status(db, model, status).await
    }

    pub async fn update_workspace_invitation_status_by_email(
        db: &DbConn,
        email_address: EmailAddress,
        status: InvitationStatus,
    ) -> Result<workspace_invitation::Model, MutationError> {
        // Query
        let model = workspace_invitation::Entity::find()
            .filter(workspace_invitation::Column::EmailAddress.eq(email_address))
            .one(db)
            .await?;
        let Some(model) = model else {
            return Err(MutationError::EntityNotFound {
                entity_name: stringify!(workspace_invitation::Entity),
            });
        };

        // Update
        Self::update_workspace_invitation_status(db, model, status).await
    }

    pub async fn update_workspace_invitation_status(
        db: &DbConn,
        model: workspace_invitation::Model,
        status: InvitationStatus,
    ) -> Result<workspace_invitation::Model, MutationError> {
        let mut active = model.into_active_model();
        active.status = Set(status);
        let model = active.update(db).await?;

        Ok(model)
    }

    pub async fn resend_workspace_invitation(
        db: &DbConn,
        uuid: &Uuid,
        model: workspace_invitation::Model,
    ) -> Result<workspace_invitation::Model, MutationError> {
        let mut active = model.into_active_model();
        active.accept_token = Set(uuid.new_v4());
        active.accept_token_expires_at = Set(Utc::now()
            .checked_add_signed(DEFAULT_WORKSPACE_INVITATION_ACCEPT_TOKEN_LIFETIME)
            .unwrap());
        let model = active.update(db).await?;

        Ok(model)
    }

    /// Create the user in database but NOT on the XMPP server.
    /// Use `UserFactory` instead, to create users in both places at the same time.
    pub async fn create_user<'a, C: ConnectionTrait>(
        db: &C,
        jid: &JID,
        role: &Option<MemberRole>,
    ) -> Result<member::Model, MutationError> {
        let mut new_member = member::ActiveModel::new();
        new_member.set_jid(jid);
        new_member.role = role.map(Set).unwrap_or(NotSet);
        new_member.insert(db).await.map_err(Into::into)
    }

    /// Accept a user invitation (i.e. delete it from database).
    /// To also create the associated user at the same time, use `UserFactory`.
    pub async fn accept_workspace_invitation<'a, C: ConnectionTrait>(
        db: &C,
        invitation: workspace_invitation::Model,
    ) -> Result<(), MutationError> {
        invitation.delete(db).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum MutationError {
    DbErr(DbErr),
    EntityNotFound { entity_name: &'static str },
}

impl fmt::Display for MutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DbErr(err) => write!(f, "Database error: {err}"),
            Self::EntityNotFound { entity_name } => write!(f, "Entity not found: {entity_name}"),
        }
    }
}

impl std::error::Error for MutationError {}

impl From<DbErr> for MutationError {
    fn from(value: DbErr) -> Self {
        Self::DbErr(value)
    }
}
