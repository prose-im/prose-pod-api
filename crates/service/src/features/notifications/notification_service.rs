// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use tracing::instrument;

use crate::{
    app_config::ConfigBranding,
    sea_orm::{prelude::*, DatabaseConnection},
};

use super::{
    dependencies::{notifier::Notification, Notifier},
    NotificationCreateForm, NotificationRepository,
};

pub struct NotificationService {
    db: DatabaseConnection,
    notifier: Arc<Notifier>,
    branding: Arc<ConfigBranding>,
}

impl NotificationService {
    pub fn new(
        db: DatabaseConnection,
        notifier: Arc<Notifier>,
        branding: Arc<ConfigBranding>,
    ) -> Self {
        Self {
            db,
            notifier,
            branding,
        }
    }

    #[instrument(
        level = "trace",
        skip(self, notification),
        fields(template = notification.template().to_string()),
        err,
    )]
    pub async fn send(&self, notification: &Notification) -> Result<(), Error> {
        // Store in DB
        let model = NotificationRepository::create(
            &self.db,
            NotificationCreateForm {
                content: notification,
                created_at: None,
            },
        )
        .await?;

        // Try sending
        if let Err(err) = self.notifier.dispatch(&self.branding, notification) {
            // TODO: Store status if undelivered
            return Err(Error::CouldNotDispatch(err));
        };

        // Delete if delivered
        NotificationRepository::delete(&self.db, model.id).await?;

        Ok(())
    }
}

pub type Error = NotificationServiceError;

#[derive(Debug, thiserror::Error)]
pub enum NotificationServiceError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("Could not dispatch notification: {0}")]
    CouldNotDispatch(String),
}
