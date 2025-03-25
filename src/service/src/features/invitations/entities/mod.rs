// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod workspace_invitation;

use crate::sea_orm_string_enum;

use super::{InvitationChannel, InvitationStatus};

sea_orm_string_enum!(InvitationStatus);
sea_orm_string_enum!(InvitationChannel);
