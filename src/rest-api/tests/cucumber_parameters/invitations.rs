// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{ops::Deref, str::FromStr};

use cucumber::Parameter;

// ===== Invitation status =====

type InvitationStatusModel = service::invitations::InvitationStatus;

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
