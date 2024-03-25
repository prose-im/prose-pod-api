// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;
use entity::model::member_invite;

// ===== Invitation channel =====

#[derive(Debug, Parameter)]
#[param(name = "invitation_channel", regex = r"Email")]
pub struct MemberInvitationChannel(member_invite::MemberInvitationChannel);

impl Deref for MemberInvitationChannel {
    type Target = member_invite::MemberInvitationChannel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MemberInvitationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Email" => Ok(Self(member_invite::MemberInvitationChannel::Email)),
            s => Err(format!("Invalid `MemberInvitationChannel`: {s}")),
        }
    }
}

// ===== Invitation status =====

#[derive(Debug, Parameter)]
#[param(name = "invitation_status", regex = r"[A-Z_]+")]
pub struct MemberInviteState(pub member_invite::MemberInviteState);

impl Deref for MemberInviteState {
    type Target = member_invite::MemberInviteState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MemberInviteState {
    type Err = <member_invite::MemberInviteState as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        member_invite::MemberInviteState::from_str(s).map(MemberInviteState)
    }
}
