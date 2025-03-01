// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{prelude::*, QueryOrder as _};

use super::entities::workspace::{ActiveModel, Column, Entity, Model};

pub enum WorkspaceRepository {}

impl WorkspaceRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<ActiveModel>,
    ) -> Result<Model, DbErr> {
        form.into().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }
}
