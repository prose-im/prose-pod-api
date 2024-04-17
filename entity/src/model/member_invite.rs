// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;
use std::str::FromStr;

use sea_orm::sea_query::{self, ArrayType, Value, ValueTypeErr};
use sea_orm::{ColumnType, TryGetError};
use serde::{Deserialize, Serialize};

use super::EmailAddress;

// ===== INVITE STATE =====

const INVITE_STATE_TO_SEND: &'static str = "TO_SEND";
const INVITE_STATE_SENT: &'static str = "SENT";
const INVITE_STATE_SEND_FAILED: &'static str = "SEND_FAILURE";

// NOTE: When adding a new case to this enum, make sure to update
//   the `column_type` function in `impl sea_query::ValueType`
//   and add a new migration to update the column size.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberInviteState {
    ToSend,
    Sent,
    SendFailed,
}

impl MemberInviteState {
    pub fn iterator() -> impl Iterator<Item = Self> {
        [
            Self::ToSend,
            Self::Sent,
            Self::SendFailed,
        ]
        .iter()
        .copied()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ToSend => INVITE_STATE_TO_SEND,
            Self::Sent => INVITE_STATE_SENT,
            Self::SendFailed => INVITE_STATE_SEND_FAILED,
        }
    }
}

impl Default for MemberInviteState {
    fn default() -> Self {
        Self::ToSend
    }
}

impl Display for MemberInviteState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<MemberInviteState> for sea_query::Value {
    fn from(value: MemberInviteState) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}

impl FromStr for MemberInviteState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            INVITE_STATE_TO_SEND => Ok(Self::ToSend),
            INVITE_STATE_SENT => Ok(Self::Sent),
            INVITE_STATE_SEND_FAILED => Ok(Self::SendFailed),
            s => Err(format!("Unknown member invite state value: {:?}", s)),
        }
    }
}

impl TryFrom<String> for MemberInviteState {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl sea_orm::TryGetable for MemberInviteState {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::prelude::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        Self::try_from(value)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| TryGetError::Null(format!("{:?}", e)))
    }
}

impl sea_query::ValueType for MemberInviteState {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => (*value).try_into().map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(MemberInviteState).to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::string(Some(
            Self::iterator().map(|v| v.to_string().len()).max().unwrap() as u32,
        ))
    }
}

// ===== INVITATION CHANNEL =====

const INVITATION_CHANNEL_EMAIL: &'static str = "EMAIL";

// NOTE: When adding a new case to this enum, make sure to update
//   the `column_type` function in `impl sea_query::ValueType`
//   and add a new migration to update the column size.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberInvitationChannel {
    Email,
}

impl MemberInvitationChannel {
    pub fn iterator() -> impl Iterator<Item = Self> {
        [Self::Email].iter().copied()
    }
}

impl ToString for MemberInvitationChannel {
    fn to_string(&self) -> String {
        match self {
            Self::Email => INVITATION_CHANNEL_EMAIL,
        }
        .to_string()
    }
}

impl From<MemberInvitationChannel> for sea_query::Value {
    fn from(value: MemberInvitationChannel) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}

impl FromStr for MemberInvitationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            INVITATION_CHANNEL_EMAIL => Ok(Self::Email),
            s => Err(format!("Unknown member invitation channel value: {:?}", s)),
        }
    }
}

impl TryFrom<String> for MemberInvitationChannel {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl sea_orm::TryGetable for MemberInvitationChannel {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::prelude::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        Self::try_from(value)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| TryGetError::Null(format!("{:?}", e)))
    }
}

impl sea_query::ValueType for MemberInvitationChannel {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => (*value).try_into().map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(MemberInvitationChannel).to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::string(Some(
            Self::iterator().map(|v| v.to_string().len()).max().unwrap() as u32,
        ))
    }
}

// ===== INVITE CONTACT =====

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "channel")]
pub enum MemberInviteContact {
    Email { email_address: EmailAddress },
}
