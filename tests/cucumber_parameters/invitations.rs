// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;

// ===== Invitation channel =====

type InvitationChannelModel = service::model::InvitationChannel;

#[derive(Debug, Parameter)]
#[param(name = "invitation_channel", regex = r"Email")]
pub struct InvitationChannel(InvitationChannelModel);

impl Deref for InvitationChannel {
    type Target = InvitationChannelModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for InvitationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Email" => Ok(Self(InvitationChannelModel::Email)),
            s => Err(format!("Invalid `InvitationChannel`: {s}")),
        }
    }
}

// ===== Invitation status =====

type InvitationStatusModel = service::model::InvitationStatus;

#[derive(Debug, Parameter)]
#[param(name = "invitation_status", regex = r"[A-Z_]+")]
pub struct InvitationStatus(pub InvitationStatusModel);

impl Deref for InvitationStatus {
    type Target = InvitationStatusModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for InvitationStatus {
    type Err = <InvitationStatusModel as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InvitationStatusModel::from_str(s).map(InvitationStatus)
    }
}
