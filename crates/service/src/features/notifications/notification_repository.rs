// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use sea_orm::{prelude::*, DeleteResult, QueryOrder as _, Set};

use super::{
    dependencies::notifier::Notification,
    entities::notification::{ActiveModel, Column, Entity, Model},
};

pub enum NotificationRepository {}

impl NotificationRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<NotificationCreateForm<'_>>,
    ) -> Result<Model, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    pub async fn next(db: &impl ConnectionTrait) -> Result<Option<Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
    }

    pub async fn delete(db: &impl ConnectionTrait, id: i32) -> Result<DeleteResult, DbErr> {
        Entity::delete_by_id(id).exec(db).await
    }
}

#[derive(Debug, Clone)]
pub struct NotificationCreateForm<'a> {
    pub content: &'a Notification,
    pub created_at: Option<DateTime<Utc>>,
}

impl<'a> NotificationCreateForm<'a> {
    fn into_active_model(self) -> ActiveModel {
        let mut res = ActiveModel {
            created_at: Set(self.created_at.unwrap_or_else(Utc::now)),
            ..Default::default()
        };
        res.set_content(&self.content);
        res
    }
}
