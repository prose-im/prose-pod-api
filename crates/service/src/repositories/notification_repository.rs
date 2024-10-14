// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::entity::notification::{ActiveModel, Column, Entity, Model};
use chrono::{DateTime, Utc};
use sea_orm::{prelude::*, QueryOrder as _, Set};

use crate::notifier::Notification;

pub enum NotificationRepository {}

impl NotificationRepository {
    pub async fn create(
        db: &impl ConnectionTrait,
        form: impl Into<NotificationCreateForm<'_>>,
    ) -> Result<Model, DbErr> {
        form.into().into_active_model().insert(db).await
    }

    pub async fn get(db: &impl ConnectionTrait) -> Result<Option<Model>, DbErr> {
        Entity::find().order_by_asc(Column::Id).one(db).await
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
