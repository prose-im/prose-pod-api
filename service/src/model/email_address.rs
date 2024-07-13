// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, ops::Deref, str::FromStr};

use sea_orm::sea_query;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, Eq, Hash, PartialEq, SerializeDisplay, DeserializeFromStr)]
pub struct EmailAddress(email_address::EmailAddress);

impl Deref for EmailAddress {
    type Target = email_address::EmailAddress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for EmailAddress {
    type Err = email_address::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        email_address::EmailAddress::from_str(s).map(|e| EmailAddress(e))
    }
}

impl From<EmailAddress> for sea_query::Value {
    fn from(value: EmailAddress) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}

impl sea_orm::TryGetable for EmailAddress {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::prelude::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let value: String = res.try_get_by(index).map_err(sea_orm::TryGetError::DbErr)?;
        Self::from_str(value.as_str())
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| sea_orm::TryGetError::Null(format!("{:?}", e)))
    }
}

impl sea_query::ValueType for EmailAddress {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(value)) => {
                Self::from_str(value.as_str()).map_err(|_| sea_query::ValueTypeErr)
            }
            _ => Err(sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(EmailAddress).to_string()
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::String
    }

    fn column_type() -> sea_orm::ColumnType {
        sea_orm::ColumnType::string(None)
    }
}

impl sea_query::Nullable for EmailAddress {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}
