// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use service::{
    invitations::{self, InvitationContact, InvitationStatus},
    members::MemberRole,
    models::BareJid,
    util::to_bare_jid,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceInvitation {
    pub invitation_id: i32,
    pub created_at: DateTime<Utc>,
    pub status: InvitationStatus,
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
    pub contact: InvitationContact,
    pub accept_token_expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceInvitationBasicDetails {
    pub jid: BareJid,
    pub pre_assigned_role: MemberRole,
}

// BOILERPLATE

impl From<invitations::entities::Invitation> for WorkspaceInvitation {
    fn from(value: invitations::entities::Invitation) -> Self {
        Self {
            invitation_id: value.id,
            created_at: value.created_at,
            status: value.status,
            jid: to_bare_jid(&value.jid).unwrap(),
            pre_assigned_role: value.pre_assigned_role,
            contact: value.contact(),
            accept_token_expires_at: value.accept_token_expires_at,
        }
    }
}

impl From<invitations::entities::Invitation> for WorkspaceInvitationBasicDetails {
    fn from(value: invitations::entities::Invitation) -> Self {
        Self {
            jid: to_bare_jid(&value.jid).unwrap(),
            pre_assigned_role: value.pre_assigned_role,
        }
    }
}
