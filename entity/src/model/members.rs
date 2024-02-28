// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp;

use sea_orm::sea_query::{self, ArrayType, Value, ValueTypeErr};
use sea_orm::{ColumnType, TryGetError};
use serde::{Deserialize, Serialize};

const MEMBER_ROLE_VALUE: &'static str = "MEMBER";
const ADMIN_ROLE_VALUE: &'static str = "ADMIN";

// NOTE: When adding a new case to this enum, make sure to update
//   the `column_type` function in `impl sea_query::ValueType`
//   and add a new migration to update the column size.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRole {
    Member,
    Admin,
}

impl Default for MemberRole {
    fn default() -> Self {
        Self::Member
    }
}

impl ToString for MemberRole {
    fn to_string(&self) -> String {
        match self {
            Self::Member => MEMBER_ROLE_VALUE.to_string(),
            Self::Admin => ADMIN_ROLE_VALUE.to_string(),
        }
    }
}

impl From<MemberRole> for sea_query::Value {
    fn from(value: MemberRole) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}

impl TryFrom<String> for MemberRole {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            MEMBER_ROLE_VALUE => Ok(Self::Member),
            ADMIN_ROLE_VALUE => Ok(Self::Admin),
            s => Err(format!("Unknown member role value: {:?}", s)),
        }
    }
}

impl sea_orm::TryGetable for MemberRole {
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

impl sea_query::ValueType for MemberRole {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => (*value).try_into().map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(MemberRole).to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::string(Some(
            cmp::max(MEMBER_ROLE_VALUE.len(), ADMIN_ROLE_VALUE.len()) as u32,
        ))
    }
}
