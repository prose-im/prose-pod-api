// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::{prelude::*, settings};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn settings(db: &DbConn) -> Result<Option<settings::Model>, DbErr> {
        Settings::find()
            .order_by_asc(settings::Column::Id)
            .one(db)
            .await
    }
}
