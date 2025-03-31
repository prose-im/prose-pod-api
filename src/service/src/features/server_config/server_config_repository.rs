// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{prelude::*, IntoActiveModel, QueryOrder as _, Set};
use tracing::instrument;

use crate::{
    models::JidDomain,
    server_config::entities::server_config::{self, ActiveModel, Column, Entity},
};

pub enum ServerConfigRepository {}

impl ServerConfigRepository {
    #[instrument(
        name = "db::server_config::is_initialized",
        level = "trace",
        skip_all,
        err
    )]
    pub async fn is_initialized(db: &impl ConnectionTrait) -> Result<bool, DbErr> {
        Ok(Entity::find().count(db).await? > 0)
    }

    #[instrument(name = "db::server_config::create", level = "trace", skip_all, err)]
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<ServerConfigCreateForm>,
    ) -> Result<server_config::Model, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    #[instrument(name = "db::server_config::get", level = "trace", skip_all, err)]
    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<server_config::Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }

    #[instrument(name = "db::server_config::reset", level = "trace", skip_all, err)]
    pub async fn reset(db: &impl ConnectionTrait) -> Result<(), DbErr> {
        let Some(model) = Self::get(db).await? else {
            return Ok(());
        };
        model.into_active_model().reset_all().update(db).await?;
        Ok(())
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
