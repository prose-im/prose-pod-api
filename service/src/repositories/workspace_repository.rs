// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::entity::workspace::{ActiveModel, Column, Entity};
use sea_orm::{prelude::*, QueryOrder as _, Set, Unchanged};

use crate::model::Workspace;

pub enum WorkspaceRepository {}

impl WorkspaceRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<ActiveModel>,
    ) -> Result<Workspace, DbErr> {
        form.into().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<Workspace>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }

    // TODO: Delete this function, as the data should be stored in the server vCard
    pub async fn set_icon_url(db: &impl ConnectionTrait, url: Option<String>) -> Result<(), DbErr> {
        let form = ActiveModel {
            id: Unchanged(1),
            icon_url: Set(url),
            ..Default::default()
        };
        form.update(db).await?;
        Ok(())
    }
}
