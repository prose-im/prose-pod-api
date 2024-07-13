// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::entity::workspace::{ActiveModel, Column, Entity};
use sea_orm::{prelude::*, NotSet, QueryOrder as _, Set, Unchanged};

use crate::model::Workspace;

pub enum WorkspaceRepository {}

impl WorkspaceRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<WorkspaceCreateForm>,
    ) -> Result<Workspace, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<Workspace>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }

    // TODO: Delete this function, as the data should be stored in the server vCard
    pub async fn set_name(db: &impl ConnectionTrait, name: String) -> Result<(), DbErr> {
        let form = ActiveModel {
            id: Unchanged(1),
            name: Set(name),
            ..Default::default()
        };
        form.update(db).await?;
        Ok(())
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

#[derive(Debug, Clone)]
pub struct WorkspaceCreateForm {
    pub name: String,
    pub accent_color: Option<Option<String>>,
}

impl WorkspaceCreateForm {
    fn into_active_model(self) -> ActiveModel {
        ActiveModel {
            name: Set(self.name),
            accent_color: self.accent_color.map(Set).unwrap_or(NotSet),
            ..Default::default()
        }
    }
}
