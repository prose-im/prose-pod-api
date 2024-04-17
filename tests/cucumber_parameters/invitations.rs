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
pub struct InvitationChannel(model::InvitationChannel);

impl Deref for InvitationChannel {
    type Target = model::InvitationChannel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for InvitationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Email" => Ok(Self(model::InvitationChannel::Email)),
            s => Err(format!("Invalid `InvitationChannel`: {s}")),
        }
    }
}

// ===== Invitation status =====

#[derive(Debug, Parameter)]
#[param(name = "invitation_status", regex = r"[A-Z_]+")]
pub struct InvitationStatus(pub model::InvitationStatus);

impl Deref for InvitationStatus {
    type Target = model::InvitationStatus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for InvitationStatus {
    type Err = <model::InvitationStatus as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        model::InvitationStatus::from_str(s).map(InvitationStatus)
    }
}
