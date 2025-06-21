// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Data Transfer Objects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use service::{
    invitations::{self, InvitationContact, InvitationStatus},
    members::MemberRole,
    models::BareJid,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceInvitationDto {
    pub invitation_id: i32,
    pub created_at: DateTime<Utc>,
    pub status: InvitationStatus,
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
    pub accept_token_expires_at: DateTime<Utc>,
}

// BOILERPLATE

impl From<invitations::entities::workspace_invitation::Model> for WorkspaceInvitationDto {
    fn from(value: invitations::entities::workspace_invitation::Model) -> Self {
        Self {
            invitation_id: value.id,
            created_at: value.created_at,
            status: value.status,
            jid: value.jid.clone().into(),
            pre_assigned_role: value.pre_assigned_role,
            contact: value.contact(),
            accept_token_expires_at: value.accept_token_expires_at,
        }
    }
}
