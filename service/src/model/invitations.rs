// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::sea_query;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{EnumIter, EnumString, IntoEnumIterator as _};

use crate::sea_orm_string_enum;

use super::EmailAddress;

// ===== INVITATION STATUS =====

// NOTE: When adding a new case to this enum, make sure to
//   add a new migration to update the column size.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(SerializeDisplay, DeserializeFromStr)]
#[derive(EnumIter, EnumString, strum::Display)]
pub enum InvitationStatus {
    #[strum(serialize = "TO_SEND")]
    ToSend,
    #[strum(serialize = "SENT")]
    Sent,
    #[strum(serialize = "SEND_FAILED")]
    SendFailed,
}

impl Default for InvitationStatus {
    fn default() -> Self {
        Self::ToSend
    }
}

sea_orm_string_enum!(InvitationStatus);

// ===== INVITATION CHANNEL =====

// NOTE: When adding a new case to this enum, make sure to
//   add a new migration to update the column size.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(EnumIter, EnumString, strum::Display)]
#[derive(SerializeDisplay, DeserializeFromStr)]
pub enum InvitationChannel {
    #[strum(serialize = "email")]
    Email,
}

sea_orm_string_enum!(InvitationChannel);

// ===== INVITATION CONTACT =====

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "channel", rename_all = "camelCase")]
pub enum InvitationContact {
    Email { email_address: EmailAddress },
}
