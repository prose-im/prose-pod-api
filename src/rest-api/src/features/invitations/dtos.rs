// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Data Transfer Objects.

use serdev::Serialize;
use service::{
    invitations::{self, InvitationContact, InvitationId, InvitationToken},
    members::MemberRole,
    models::BareJid,
};
use time::OffsetDateTime;

#[derive(Debug)]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
pub struct WorkspaceInvitationDto {
    pub invitation_id: InvitationId,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    pub jid: BareJid,

    pub pre_assigned_role: MemberRole,

    pub contact: InvitationContact,

    pub accept_token: InvitationToken,
    pub reject_token: InvitationToken,

    #[serde(with = "time::serde::rfc3339")]
    pub accept_token_expires_at: OffsetDateTime,

    pub is_expired: bool,
}

// MARK: - Boilerplate

impl From<invitations::Invitation> for WorkspaceInvitationDto {
    fn from(value: invitations::Invitation) -> Self {
        Self {
            is_expired: value.is_expired(),
            invitation_id: value.id.clone(),
            created_at: value.created_at,
            jid: value.jid.clone().into(),
            pre_assigned_role: value.pre_assigned_role,
            contact: value.contact(),
            accept_token: value.accept_token,
            reject_token: value.reject_token,
            accept_token_expires_at: value.accept_token_expires_at,
        }
    }
}
