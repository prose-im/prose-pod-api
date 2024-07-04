// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use entity::server_config::{ActiveModel, Column, Entity, Model};
use sea_orm::{prelude::*, QueryOrder as _, Set};

pub type ServerConfig = Model;

pub enum ServerConfigRepository {}

impl ServerConfigRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<ServerConfigCreateForm>,
    ) -> Result<Model, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfigCreateForm {
    pub domain: String,
}

impl ServerConfigCreateForm {
    fn into_active_model(self) -> ActiveModel {
        ActiveModel {
            domain: Set(self.domain),
            ..Default::default()
        }
    }
}
