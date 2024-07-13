use std::{fmt::Display, ops::Deref, str::FromStr};

use jid::NodePart;
use sea_orm::{sea_query, TryGetError};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::EmailAddress;

#[derive(Debug, Clone, Eq, Hash, PartialEq, DeserializeFromStr, SerializeDisplay)]
#[repr(transparent)]
pub struct JidNode(NodePart);

impl Deref for JidNode {
    type Target = <NodePart as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl FromStr for JidNode {
    type Err = <NodePart as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NodePart::from_str(s).map(Self)
    }
}

impl Display for JidNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<EmailAddress> for JidNode {
    fn from(value: EmailAddress) -> Self {
        // NOTE: Email adresses are already parsed, and their local part are equivalent to a JID node part.
        Self::from_str(value.local_part()).unwrap()
    }
}

impl From<JidNode> for sea_query::Value {
    fn from(value: JidNode) -> Self {
        Self::String(Some(Box::new(value.to_string())))
    }
}

impl sea_orm::TryGetable for JidNode {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::prelude::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        Self::from_str(&value)
            // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
            .map_err(|e| TryGetError::Null(format!("{:?}", e)))
    }
}

impl sea_query::ValueType for JidNode {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(value)) => {
                Self::from_str(value.as_str()).map_err(|_| sea_query::ValueTypeErr)
            }
            _ => Err(sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(JidNode).to_string()
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::String
    }

    fn column_type() -> sea_orm::ColumnType {
        sea_orm::ColumnType::string(None)
    }
}

impl sea_query::Nullable for JidNode {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}
