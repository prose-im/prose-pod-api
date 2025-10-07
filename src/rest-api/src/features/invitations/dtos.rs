// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Data Transfer Objects.

use chrono::{DateTime, Utc};
use serdev::Serialize;
use service::{
    invitations::{self, InvitationContact, InvitationId},
    members::MemberRole,
    models::BareJid,
};

#[derive(Debug)]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
pub struct WorkspaceInvitationDto {
    pub invitation_id: InvitationId,
    pub created_at: DateTime<Utc>,
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
    pub accept_token_expires_at: DateTime<Utc>,
    pub is_expired: bool,
}

// MARK: - Boilerplate

impl From<invitations::Invitation> for WorkspaceInvitationDto {
    fn from(value: invitations::Invitation) -> Self {
        Self {
            invitation_id: value.id.clone(),
            created_at: value.created_at,
            jid: value.jid.clone().into(),
            pre_assigned_role: value.pre_assigned_role,
            contact: value.contact(),
            accept_token_expires_at: value.accept_token_expires_at,
            is_expired: value.is_expired(),
        }
    }
}
