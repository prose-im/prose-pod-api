// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set, Unchanged,
};
use serde_json::Value as Json;
use tracing::{instrument, warn};

use crate::util::either::Either;

pub use self::entity::Model as KvRecord;
use self::entity::{ActiveModel, Column, Entity};

mod entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "kv_store")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub namespace: String,
        #[sea_orm(primary_key, auto_increment = false)]
        pub key: String,
        pub value: Json,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(Debug)]
pub struct KvStore;

impl KvStore {
    #[instrument(name = "store::set", level = "trace", skip_all, fields(namespace, key))]
    pub async fn set(
        db: &impl ConnectionTrait,
        namespace: &str,
        key: &str,
        value: Json,
    ) -> Result<(), DbErr> {
        let primary_key = (namespace.to_owned(), key.to_owned());
        if Entity::find_by_id(primary_key).count(db).await? > 0 {
            ActiveModel {
                namespace: Unchanged(namespace.to_owned()),
                key: Unchanged(key.to_owned()),
                value: Set(value),
            }
            .update(db)
            .await
        } else {
            ActiveModel {
                namespace: Set(namespace.to_owned()),
                key: Set(key.to_owned()),
                value: Set(value),
            }
            .insert(db)
            .await
        }
        .map(|_| ())
    }

    #[instrument(name = "store::get_all", level = "trace", skip_all, fields(namespace))]
    pub async fn get_all(
        db: &impl ConnectionTrait,
        namespace: &str,
    ) -> Result<HashMap<String, Json>, DbErr> {
        let select = Entity::find().filter(Column::Namespace.eq(namespace));
        let models = select.all(db).await?;
        Ok(models.into_iter().map(|kv| (kv.key, kv.value)).collect())
    }

    #[instrument(name = "store::get", level = "trace", skip_all, fields(namespace, key))]
    pub async fn get(
        db: &impl ConnectionTrait,
        namespace: &str,
        key: &str,
    ) -> Result<Option<Json>, DbErr> {
        let primary_key = (namespace.to_owned(), key.to_owned());
        let select = Entity::find_by_id(primary_key);
        match select.one(db).await {
            Ok(Some(kv)) => Ok(Some(kv.value)),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Returns whether or not a record was deleted.
    #[instrument(
        name = "store::delete",
        level = "trace",
        skip_all,
        fields(namespace, key)
    )]
    pub async fn delete(
        db: &impl ConnectionTrait,
        namespace: &str,
        key: &str,
    ) -> Result<bool, DbErr> {
        let primary_key = (namespace.to_owned(), key.to_owned());
        let select = Entity::delete_by_id(primary_key);
        match select.exec(db).await {
            Ok(sea_orm::DeleteResult { rows_affected: 0 }) => Ok(false),
            Ok(_) => Ok(true),
            Err(err) => Err(err),
        }
    }
}

// MARK: Typed getters/setters

macro_rules! get_set {
    (
        $t:ty,
        $get_fn:ident as $get_fn_span:literal,
        $set_fn:ident as $set_fn_span:literal
        $(,)?
    ) => {
        #[tracing::instrument(
            name = $get_fn_span,
            level = "trace",
            skip_all,
            fields(namespace, key),
        )]
        pub async fn $get_fn(
            db: &impl ConnectionTrait,
            namespace: &str,
            key: &str,
        ) -> Result<Option<$t>, DbErr> {
            match Self::get(db, namespace, key).await {
                Ok(Some(json)) => Ok(serde_json::from_value(json)
                    .inspect_err(|err| {
                        warn!(
                            "JSON value for `{namespace}`/`{key}` could not be parsed to `{type}`: {err}",
                            type = stringify!($t),
                        )
                    })
                    .ok()),
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            }
        }

        #[tracing::instrument(
            name = $set_fn_span,
            level = "trace",
            skip_all,
            fields(namespace, key),
        )]
        pub async fn $set_fn(
            db: &impl ConnectionTrait,
            namespace: &str,
            key: &str,
            value: $t,
        ) -> Result<(), Either<serde_json::Error, DbErr>> {
            let json = serde_json::to_value(value).map_err(Either::E1)?;
            (Self::set(db, namespace, key, json).await).map_err(Either::E2)
        }
    };
}

impl KvStore {
    get_set!(
        bool,
        get_bool as "store::get_bool",
        set_bool as "store::set_bool",
    );
    get_set!(
        String,
        get_string as "store::get_string",
        set_string as "store::set_string",
    );
}

// MARK: Scoped store generator

#[macro_export]
#[doc(hidden)]
macro_rules! kv_store_scoped_get_set {
    (bool) => {
        crate::kv_store_scoped_get_set!(bool, get_bool, set_bool);
    };
    (string) => {
        crate::kv_store_scoped_get_set!(String, get_string, set_string);
    };
    (
        $t:ty,
        $get_fn:ident,
        $set_fn:ident
    ) => {
        #[allow(unused)]
        pub async fn $get_fn(db: &impl ConnectionTrait, key: &str) -> anyhow::Result<Option<$t>> {
            (global_storage::KvStore::$get_fn(db, NAMESPACE, key).await)
                .map_err(|err| anyhow::anyhow!("Database error: {err}"))
        }

        #[allow(unused)]
        pub async fn $set_fn(
            db: &impl ConnectionTrait,
            key: &str,
            value: $t,
        ) -> anyhow::Result<()> {
            use crate::util::either::Either;
            (global_storage::KvStore::$set_fn(db, NAMESPACE, key, value).await).map_err(|err| {
                match err {
                    Either::E1(err) => anyhow::anyhow!("JSON error: {err}"),
                    Either::E2(err) => anyhow::anyhow!("Database error: {err}"),
                }
            })
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! gen_scoped_kv_store {
    ($namespace:literal$(; get/set: $($impl:ident)+)?) => {
        #[doc(hidden)]
        mod kv_store {
            use std::collections::HashMap;

            use sea_orm::ConnectionTrait;
            use serde_json::Value as Json;

            use crate::global_storage;

            const NAMESPACE: &'static str = $namespace;

            #[derive(Debug)]
            pub struct KvStore;

            impl KvStore {
                #[allow(unused)]
                pub async fn set(
                    db: &impl ConnectionTrait,
                    key: &str,
                    value: Json,
                ) -> anyhow::Result<()> {
                    (global_storage::KvStore::set(db, NAMESPACE, key, value).await)
                        .map_err(|err| anyhow::anyhow!("Database error: {err}"))
                }

                #[allow(unused)]
                pub async fn get_all(
                    db: &impl ConnectionTrait,
                ) -> anyhow::Result<HashMap<String, Json>> {
                    (global_storage::KvStore::get_all(db, NAMESPACE).await)
                        .map_err(|err| anyhow::anyhow!("Database error: {err}"))
                }

                #[allow(unused)]
                pub async fn get(
                    db: &impl ConnectionTrait,
                    key: &str,
                ) -> anyhow::Result<Option<Json>> {
                    (global_storage::KvStore::get(db, NAMESPACE, key).await)
                        .map_err(|err| anyhow::anyhow!("Database error: {err}"))
                }

                /// Returns whether or not a record was deleted.
                #[allow(unused)]
                pub async fn delete(
                    db: &impl ConnectionTrait,
                    key: &str,
                ) -> anyhow::Result<bool> {
                    (global_storage::KvStore::delete(db, NAMESPACE, key).await)
                        .map_err(|err| anyhow::anyhow!("Database error: {err}"))
                }
            }

            $(impl KvStore {
                $(crate::kv_store_scoped_get_set!($impl);)+
            })?
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! gen_kv_store_get_set {
    ($val:ident: bool) => {
        pub mod $val {
            use sea_orm::ConnectionTrait;

            use super::KvStore;

            #[tracing::instrument(level = "trace", skip_all)]
            pub async fn get_opt(db: &impl ConnectionTrait) -> Option<bool> {
                (KvStore::get_bool(db, stringify!($val)).await).unwrap_or(None)
            }

            #[tracing::instrument(level = "trace", skip_all)]
            pub async fn get(db: &impl ConnectionTrait) -> bool {
                (KvStore::get_bool(db, stringify!($val)).await)
                    .unwrap_or(None)
                    .unwrap_or(false)
            }

            #[tracing::instrument(level = "trace", skip_all, fields(new_value))]
            pub async fn set(db: &impl ConnectionTrait, new_value: bool) -> anyhow::Result<()> {
                KvStore::set_bool(db, stringify!($val), new_value).await
            }
        }
    };
}
