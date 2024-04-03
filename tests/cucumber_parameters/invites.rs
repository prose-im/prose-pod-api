// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;
use entity::model;

// ===== Invitation channel =====

#[derive(Debug, Parameter)]
#[param(name = "invitation_channel", regex = r"Email")]
pub struct MemberInvitationChannel(model::MemberInvitationChannel);

impl Deref for MemberInvitationChannel {
    type Target = model::MemberInvitationChannel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MemberInvitationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Email" => Ok(Self(model::MemberInvitationChannel::Email)),
            s => Err(format!("Invalid `MemberInvitationChannel`: {s}")),
        }
    }
}

// ===== Invitation status =====

#[derive(Debug, Parameter)]
#[param(name = "invitation_status", regex = r"[A-Z_]+")]
pub struct MemberInviteState(pub model::MemberInviteState);

impl Deref for MemberInviteState {
    type Target = model::MemberInviteState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MemberInviteState {
    type Err = <model::MemberInviteState as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        model::MemberInviteState::from_str(s).map(MemberInviteState)
    }
}
