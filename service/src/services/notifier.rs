// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{path::PathBuf, str::FromStr as _, sync::Arc};

use crate::{
    config::ConfigBranding,
    dependencies,
    notifier::Notification,
    repositories::{NotificationCreateForm, NotificationRepository},
    sea_orm::{prelude::*, DatabaseConnection},
};

pub struct Notifier<'r> {
    db: &'r DatabaseConnection,
    notifier: &'r dependencies::Notifier,
    branding: &'r ConfigBranding,
}

impl<'r> Notifier<'r> {
    pub fn new(
        db: &'r DatabaseConnection,
        notifier: &'r dependencies::Notifier,
        branding: &'r ConfigBranding,
    ) -> Self {
        Self {
            db,
            notifier,
            branding,
        }
    }

    async fn send(&self, notification: &Notification) -> Result<(), Error> {
        // Store in DB
        NotificationRepository::create(
            self.db,
            NotificationCreateForm {
                content: notification,
                created_at: None,
            },
        )
        .await?;

        // Try sending
        self.notifier
            .dispatch(self.branding, notification)
            .map_err(|e| Error::CouldNotDispatch(e))?;

        // Store status if undelivered

        // Delete if delivered

        Ok(())
    }

    pub async fn send_workspace_invitation(
        &self,
        branding: &ConfigBranding,
        accept_token: Uuid,
        reject_token: Uuid,
    ) -> Result<(), Error> {
        let admin_site_root = PathBuf::from_str(&branding.page_url.to_string()).unwrap();
        self.send(&Notification::WorkspaceInvitation {
            accept_link: admin_site_root
                .join(format!("invitations/accept/{accept_token}"))
                .display()
                .to_string(),
            reject_link: admin_site_root
                .join(format!("invitations/reject/{reject_token}"))
                .display()
                .to_string(),
        })
        .await
    }
}

pub type Error = NotifierError;

#[derive(Debug, Clone, thiserror::Error)]
pub enum NotifierError {
    #[error("Database error: {0}")]
    DbErr(#[from] Arc<DbErr>),
    #[error("Could not dispatch notification: {0}")]
    CouldNotDispatch(String),
}

impl From<DbErr> for NotifierError {
    fn from(err: DbErr) -> Self {
        Self::DbErr(Arc::new(err))
    }
}
