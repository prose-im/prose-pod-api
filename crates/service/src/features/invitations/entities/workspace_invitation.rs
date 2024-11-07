// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use uuid::Uuid;

pub use crate::features::invitations::*;
use crate::{
    features::members::MemberRole,
    models::{EmailAddress, JID},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "workspace_invitation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub created_at: DateTimeUtc,
    pub status: InvitationStatus,
    pub jid: JID,
    pub pre_assigned_role: MemberRole,
    invitation_channel: InvitationChannel,
    email_address: Option<EmailAddress>,
    /// Expiring one-time use token used to accept an invitation.
    /// Will change every time an admin resends the invitation.
    /// Will be deleted along with the entire invitation once used.
    pub accept_token: Uuid,
    pub accept_token_expires_at: DateTimeUtc,
    /// Unique token used by someone to reject an invitation (e.g. because of
    /// misspelled email address).
    /// Never expires, will be usable as long as the invitation still exists.
    /// Will be deleted along with the entire invitation once used.
    pub reject_token: Uuid,
}

impl Model {
    pub fn contact(&self) -> InvitationContact {
        match self.invitation_channel {
            InvitationChannel::Email => InvitationContact::Email {
                email_address: self.email_address.clone().unwrap(),
            },
        }
    }
}

impl ActiveModel {
    pub fn set_contact(&mut self, contact: InvitationContact) {
        // Unset every optional contact detail in case we change channel.
        self.email_address = NotSet;

        match contact {
            InvitationContact::Email { email_address } => {
                self.invitation_channel = Set(InvitationChannel::Email);
                self.email_address = Set(Some(email_address));
            }
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
