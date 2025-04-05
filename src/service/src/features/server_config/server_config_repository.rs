// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{prelude::*, IntoActiveModel, QueryOrder as _, QuerySelect, Set, Unchanged};
use tracing::instrument;

use crate::{
    models::JidDomain,
    prosody::ProsodyOverrides,
    server_config::entities::server_config::{self, ActiveModel, Column, Entity},
    util::Either,
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

    #[instrument(
        name = "db::server_config::set_prosody_overrides",
        level = "trace",
        skip_all,
        err
    )]
    pub(crate) async fn set_prosody_overrides(
        db: &impl ConnectionTrait,
        overrides: ProsodyOverrides,
    ) -> Result<server_config::Model, Either<serde_json::Error, DbErr>> {
        let json = serde_json::to_value(overrides).map_err(Either::Left)?;
        let mut model = ActiveModel::new();
        model.id = Unchanged(1);
        model.prosody_overrides = Set(Some(json));
        model.update(db).await.map_err(Either::Right)
    }

    /// - `Ok(Some(None))` => Server config initialized, no value
    /// - `Ok(None)` => Server config not initialized
    #[instrument(
        name = "db::server_config::get_prosody_overrides",
        level = "trace",
        skip_all,
        err
    )]
    pub(crate) async fn get_prosody_overrides(
        db: &impl ConnectionTrait,
    ) -> Result<Option<Option<ProsodyOverrides>>, Either<DbErr, serde_json::Error>> {
        let json = Entity::find()
            .order_by_asc(Column::Id)
            .select_only()
            .column(Column::ProsodyOverrides)
            .into_tuple::<Option<Json>>()
            .one(db)
            .await
            .map_err(Either::Left)?;
        match json {
            Some(Some(json)) => match serde_json::from_value::<ProsodyOverrides>(json) {
                Ok(v) => Ok(Some(Some(v))),
                Err(e) => Err(Either::Right(e)),
            },
            Some(None) => Ok(Some(None)),
            None => Ok(None),
        }
    }

    #[instrument(
        name = "db::server_config::delete_prosody_overrides",
        level = "trace",
        skip_all,
        err
    )]
    pub(crate) async fn delete_prosody_overrides(
        db: &impl ConnectionTrait,
    ) -> Result<server_config::Model, DbErr> {
        let mut model = ActiveModel::new();
        model.id = Unchanged(1);
        model.prosody_overrides = Set(None);
        model.update(db).await
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
