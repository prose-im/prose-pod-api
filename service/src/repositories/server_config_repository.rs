// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::server_config::{ActiveModel, Column, Entity, Model};
use sea_orm::{prelude::*, QueryOrder as _};

pub enum ServerConfigRepository {}

impl ServerConfigRepository {
    pub async fn create<'a, C: ConnectionTrait>(
        db: &C,
        form_data: ActiveModel,
    ) -> Result<Model, DbErr> {
        form_data.insert(db).await
    }

    pub async fn get(db: &DbConn) -> Result<Option<Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }
}
