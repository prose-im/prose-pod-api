// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use crate::{
    app_config::ConfigBranding,
    sea_orm::{prelude::*, DatabaseConnection},
};

use super::{
    dependencies::{notifier::Notification, Notifier},
    NotificationCreateForm, NotificationRepository,
};

pub struct NotificationService {
    db: Arc<DatabaseConnection>,
    notifier: Arc<Notifier>,
    branding: Arc<ConfigBranding>,
}

impl NotificationService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        notifier: Arc<Notifier>,
        branding: Arc<ConfigBranding>,
    ) -> Self {
        Self {
            db,
            notifier,
            branding,
        }
    }

    pub async fn send(&self, notification: &Notification) -> Result<(), Error> {
        // Store in DB
        NotificationRepository::create(
            self.db.as_ref(),
            NotificationCreateForm {
                content: notification,
                created_at: None,
            },
        )
        .await?;

        // Try sending
        if let Err(err) = self.notifier.dispatch(&self.branding, notification) {
            // Store status if undelivered
            todo!("Store status if undelivered");

            return Err(Error::CouldNotDispatch(err));
        };

        // Delete if delivered
        todo!("Delete if delivered");

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
