// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;
use std::str::FromStr;

use sea_orm::sea_query::{self, ArrayType, Value, ValueTypeErr};
use sea_orm::{ColumnType, TryGetError};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::EmailAddress;

// ===== INVITATION STATUS =====

const INVITATION_STATUS_TO_SEND: &'static str = "TO_SEND";
const INVITATION_STATUS_SENT: &'static str = "SENT";
const INVITATION_STATUS_SEND_FAILED: &'static str = "SEND_FAILED";

// NOTE: When adding a new case to this enum, make sure to update
//   the `column_type` function in `impl sea_query::ValueType`
//   and add a new migration to update the column size.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InvitationStatus {
    ToSend,
    Sent,
    SendFailed,
}

impl InvitationStatus {
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
            Self::ToSend => INVITATION_STATUS_TO_SEND,
            Self::Sent => INVITATION_STATUS_SENT,
            Self::SendFailed => INVITATION_STATUS_SEND_FAILED,
        }
    }
}

impl Default for InvitationStatus {
    fn default() -> Self {
        Self::ToSend
    }
}

impl Display for InvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for InvitationStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl From<InvitationStatus> for sea_query::Value {
    fn from(value: InvitationStatus) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}

impl FromStr for InvitationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            INVITATION_STATUS_TO_SEND => Ok(Self::ToSend),
            INVITATION_STATUS_SENT => Ok(Self::Sent),
            INVITATION_STATUS_SEND_FAILED => Ok(Self::SendFailed),
            s => Err(format!("Unknown workspace invitation status: {:?}", s)),
        }
    }
}

impl TryFrom<String> for InvitationStatus {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl<'de> Deserialize<'de> for InvitationStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status_string = String::deserialize(deserializer)?;
        Self::try_from(status_string).map_err(|err| serde::de::Error::custom(&err))
    }
}

impl sea_orm::TryGetable for InvitationStatus {
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

impl sea_query::ValueType for InvitationStatus {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => (*value).try_into().map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(InvitationStatus).to_string()
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
pub enum InvitationChannel {
    Email,
}

impl InvitationChannel {
    pub fn iterator() -> impl Iterator<Item = Self> {
        [Self::Email].iter().copied()
    }
}

impl ToString for InvitationChannel {
    fn to_string(&self) -> String {
        match self {
            Self::Email => INVITATION_CHANNEL_EMAIL,
        }
        .to_string()
    }
}

impl From<InvitationChannel> for sea_query::Value {
    fn from(value: InvitationChannel) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}

impl FromStr for InvitationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            INVITATION_CHANNEL_EMAIL => Ok(Self::Email),
            s => Err(format!(
                "Unknown workspace invitation channel value: {:?}",
                s
            )),
        }
    }
}

impl TryFrom<String> for InvitationChannel {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl sea_orm::TryGetable for InvitationChannel {
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

impl sea_query::ValueType for InvitationChannel {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => (*value).try_into().map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(InvitationChannel).to_string()
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

// ===== INVITATION CONTACT =====

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "channel", rename_all = "camelCase")]
pub enum InvitationContact {
    Email { email_address: EmailAddress },
}
