// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{path::PathBuf, str::FromStr as _, sync::Arc};

use secrecy::ExposeSecret as _;

use crate::{
    app_config::ConfigBranding,
    invitations::InvitationToken,
    sea_orm::{prelude::*, DatabaseConnection},
};

use crate::notifications::{
    dependencies, dependencies::any_notifier::Notification, NotificationCreateForm,
    NotificationRepository,
};

pub struct Notifier {
    db: Arc<DatabaseConnection>,
    notifier: Arc<dependencies::Notifier>,
    branding: Arc<ConfigBranding>,
}

impl Notifier {
    pub fn new(
        db: Arc<DatabaseConnection>,
        notifier: Arc<dependencies::Notifier>,
        branding: Arc<ConfigBranding>,
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
            self.db.as_ref(),
            NotificationCreateForm {
                content: notification,
                created_at: None,
            },
        )
        .await?;

        // Try sending
        self.notifier
            .dispatch(&self.branding, notification)
            .map_err(|e| Error::CouldNotDispatch(e))?;

        // Store status if undelivered

        // Delete if delivered

        Ok(())
    }

    pub async fn send_workspace_invitation(
        &self,
        branding: &ConfigBranding,
        accept_token: &InvitationToken,
        reject_token: &InvitationToken,
    ) -> Result<(), Error> {
        let admin_site_root = PathBuf::from_str(&branding.page_url.to_string()).unwrap();
        self.send(&Notification::WorkspaceInvitation {
            accept_link: admin_site_root
                .join(format!(
                    "invitations/accept/{}",
                    accept_token.expose_secret()
                ))
                .display()
                .to_string(),
            reject_link: admin_site_root
                .join(format!(
                    "invitations/reject/{}",
                    reject_token.expose_secret()
                ))
                .display()
                .to_string(),
        })
        .await
    }
}

pub type Error = NotifierError;

#[derive(Debug, thiserror::Error)]
pub enum NotifierError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("Could not dispatch notification: {0}")]
    CouldNotDispatch(String),
}
