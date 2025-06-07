// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[doc(hidden)]
#[macro_export]
macro_rules! sea_orm_try_get_by_string {
    (using: FromStr) => {
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
    (using: From<String>) => {
        fn try_get_by<I: sea_orm::ColIdx>(
            res: &sea_orm::QueryResult,
            index: I,
        ) -> Result<Self, sea_orm::TryGetError> {
            // https://github.com/SeaQL/sea-orm/discussions/1176#discussioncomment-4024088
            (res.try_get_by(index))
                .map_err(sea_orm::TryGetError::DbErr)
                .map(<Self as From<String>>::from)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sea_orm_string_value_impl {
    ($t:ty) => {
        fn from(value: $t) -> Self {
            Self::String(Some(Box::new(value.to_string())))
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sea_orm_string_value_type_impl {
    ($t:ty, $length:expr, using: FromStr) => {
        fn try_from(
            v: sea_orm::sea_query::Value,
        ) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
            match v {
                sea_orm::sea_query::Value::String(Some(value)) => {
                    <Self as std::str::FromStr>::from_str(value.as_str())
                        .map_err(|_| sea_orm::sea_query::ValueTypeErr)
                }
                _ => Err(sea_orm::sea_query::ValueTypeErr),
            }
        }

        fn type_name() -> String {
            stringify!($t).to_string()
        }

        fn array_type() -> sea_orm::sea_query::ArrayType {
            sea_orm::sea_query::ArrayType::String
        }

        fn column_type() -> sea_orm::sea_query::ColumnType {
            sea_orm::sea_query::ColumnType::string($length)
        }
    };
    ($t:ty, $length:expr, using: From<String>) => {
        fn try_from(
            v: sea_orm::sea_query::Value,
        ) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
            match v {
                sea_orm::sea_query::Value::String(Some(value)) => {
                    Ok(<Self as From<String>>::from(*value))
                }
                _ => Err(sea_orm::sea_query::ValueTypeErr),
            }
        }

        fn type_name() -> String {
            stringify!($t).to_string()
        }

        fn array_type() -> sea_orm::sea_query::ArrayType {
            sea_orm::sea_query::ArrayType::String
        }

        fn column_type() -> sea_orm::sea_query::ColumnType {
            sea_orm::sea_query::ColumnType::string($length)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sea_orm_string_nullable_impl {
    () => {
        fn null() -> sea_orm::Value {
            sea_orm::Value::String(None)
        }
    };
}

#[macro_export]
macro_rules! sea_orm_string {
    ($t:ty) => {
        crate::sea_orm_string!($t, length: None);
    };
    ($t:ty, length: $length:expr) => {
        impl From<$t> for sea_orm::sea_query::Value {
            crate::sea_orm_string_value_impl!($t);
        }

        impl sea_orm::TryGetable for $t {
            crate::sea_orm_try_get_by_string!(using: FromStr);
        }

        impl sea_orm::sea_query::ValueType for $t {
            crate::sea_orm_string_value_type_impl!($t, $length, using: FromStr);
        }

        impl sea_orm::sea_query::Nullable for $t {
            crate::sea_orm_string_nullable_impl!();
        }
    };
    ($t:ty; enum) => {
        crate::sea_orm_string!(
            $t,
            length: Some(
                <Self as strum::IntoEnumIterator>::iter()
                    .map(|v| v.to_string().len())
                    .max()
                    .unwrap() as u32
            )
        );
    };
    ($t:ty; secret) => {
        impl From<$t> for sea_orm::sea_query::Value {
            fn from(value: $t) -> Self {
                use secrecy::ExposeSecret;
                Self::String(Some(Box::new(value.expose_secret().to_string())))
            }
        }

        impl sea_orm::TryGetable for $t {
            crate::sea_orm_try_get_by_string!(using: From<String>);
        }

        impl sea_orm::sea_query::ValueType for $t {
            crate::sea_orm_string_value_type_impl!($t, None, using: From<String>);
        }

        impl sea_orm::sea_query::Nullable for $t {
            crate::sea_orm_string_nullable_impl!();
        }
    };
}
