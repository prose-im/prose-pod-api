//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::model::member_invite::*;

use crate::model::{
    member_invite::{MemberInvitationChannel, MemberInviteContact, MemberInviteState},
    EmailAddress, MemberRole, JID,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "member_invite")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub created_at: DateTimeUtc,
    pub state: MemberInviteState,
    pub jid: JID,
    pub pre_assigned_role: MemberRole,
    invitation_channel: MemberInvitationChannel,
    email_address: Option<EmailAddress>,
    /// Expiring one-time use token used to accept an invite.
    /// Will change every time an admin resends the invite.
    /// Will be deleted along with the entire invite once used.
    pub accept_token: Uuid,
    pub accept_token_expires_at: DateTimeUtc,
    /// Unique token used by someone to reject an invite (e.g. because of
    /// misspelled email address).
    /// Never expires, will be usable as long as the invite still exists.
    /// Will be deleted along with the entire invite once used.
    pub reject_token: Uuid,
}

impl Model {
    pub fn contact(&self) -> MemberInviteContact {
        match self.invitation_channel {
            MemberInvitationChannel::Email => MemberInviteContact::Email {
                email_address: self.email_address.clone().unwrap(),
            },
        }
    }
}

impl ActiveModel {
    pub fn set_contact(&mut self, contact: MemberInviteContact) {
        // Unset every optional contact detail in case we change channel.
        self.email_address = NotSet;

        match contact {
            MemberInviteContact::Email { email_address } => {
                self.invitation_channel = Set(MemberInvitationChannel::Email);
                self.email_address = Set(Some(email_address));
            }
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}