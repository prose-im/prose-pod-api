// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{
    features::server_config::ServerConfig,
    model::jid::{self, BareJid, NodePart},
};

pub fn to_bare_jid(jid: &crate::model::JID) -> Result<BareJid, jid::Error> {
    BareJid::new(jid.to_string().as_str())
}

pub fn bare_jid_from_username(
    username: &str,
    server_config: &ServerConfig,
) -> Result<BareJid, String> {
    Ok(BareJid::from_parts(
        Some(&NodePart::new(username).map_err(|err| format!("Invalid username: {err}"))?),
        &server_config.domain,
    ))
}

#[macro_export]
macro_rules! wrapper_type {
    ($wrapper:ident, $t:ty) => {
        #[derive(std::fmt::Debug, Clone, Eq, PartialEq, Hash)]
        #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
        #[repr(transparent)]
        pub struct $wrapper($t);

        impl std::ops::Deref for $wrapper {
            type Target = $t;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $wrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl std::str::FromStr for $wrapper {
            type Err = <$t as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$t>::from_str(s).map(Self)
            }
        }

        impl From<$t> for $wrapper {
            fn from(bare_jid: $t) -> Self {
                Self(bare_jid)
            }
        }
    };
}

#[macro_export]
macro_rules! sea_orm_try_get_by_string {
    () => {
        fn try_get_by<I: sea_orm::ColIdx>(
            res: &sea_orm::QueryResult,
            index: I,
        ) -> Result<Self, sea_orm::TryGetError> {
            // https://github.com/SeaQL/sea-orm/discussions/1176#discussioncomment-4024088
            let value = res
                .try_get_by(index)
                .map_err(sea_orm::TryGetError::DbErr)
                .and_then(|opt: Option<String>| {
                    opt.ok_or(sea_orm::TryGetError::Null(format!("{index:?}")))
                })?;
            <Self as std::str::FromStr>::from_str(value.as_str())
                // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
                .map_err(|e| sea_orm::TryGetError::Null(format!("{:?}", e)))
        }
    };
}

#[macro_export]
macro_rules! sea_orm_string_ {
    ($t:ty, $length:expr) => {
        impl From<$t> for sea_query::Value {
            fn from(value: $t) -> Self {
                Self::String(Some(Box::new(value.to_string())))
            }
        }

        impl sea_orm::TryGetable for $t {
            crate::sea_orm_try_get_by_string!();
        }

        impl sea_query::ValueType for $t {
            fn try_from(v: sea_query::Value) -> Result<Self, sea_query::ValueTypeErr> {
                match v {
                    sea_query::Value::String(Some(value)) => {
                        <Self as std::str::FromStr>::from_str(value.as_str())
                            .map_err(|_| sea_query::ValueTypeErr)
                    }
                    _ => Err(sea_query::ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($t).to_string()
            }

            fn array_type() -> sea_query::ArrayType {
                sea_query::ArrayType::String
            }

            fn column_type() -> sea_query::ColumnType {
                sea_query::ColumnType::string($length)
            }
        }

        impl sea_query::Nullable for $t {
            fn null() -> sea_orm::Value {
                sea_orm::Value::String(None)
            }
        }
    };
}

#[macro_export]
macro_rules! sea_orm_string {
    ($t:ty) => {
        crate::sea_orm_string_!($t, None);
    };
}

#[macro_export]
macro_rules! sea_orm_string_enum {
    ($t:ty) => {
        crate::sea_orm_string_!(
            $t,
            Some(Self::iter().map(|v| v.to_string().len()).max().unwrap() as u32)
        );
    };
}
