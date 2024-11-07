// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{prelude::*, QueryOrder as _, Set};

use crate::{
    features::server_config::entities::server_config::{self, ActiveModel, Column, Entity},
    models::JidDomain,
};

pub enum ServerConfigRepository {}

impl ServerConfigRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<ServerConfigCreateForm>,
    ) -> Result<server_config::Model, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<server_config::Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfigCreateForm {
    pub domain: JidDomain,
}

impl ServerConfigCreateForm {
    fn into_active_model(self) -> ActiveModel {
        ActiveModel {
            domain: Set(self.domain),
            ..Default::default()
        }
    }
}
