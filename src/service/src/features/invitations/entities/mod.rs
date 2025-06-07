// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod workspace_invitation;

use super::{InvitationChannel, InvitationStatus};

crate::sea_orm_string!(InvitationStatus; enum);
crate::sea_orm_string!(InvitationChannel; enum);
