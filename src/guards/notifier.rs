// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt, path::PathBuf, str::FromStr as _};

use chrono::Utc;
use entity::{notification, server_config};
use migration::DbErr;
use rocket::{outcome::try_outcome, request::Outcome, Request, State};
use sea_orm_rocket::Connection;
use service::{
    notifier::Notification,
    sea_orm::{prelude::*, ActiveModelBehavior, DatabaseConnection, Set},
    Query, APP_CONF,
};

use crate::error::{self, Error};

use super::{Db, FromRequest, JID as JIDGuard};

pub struct Notifier<'r> {
    db: &'r DatabaseConnection,
    notifier: &'r State<service::dependencies::Notifier>,
    server_config: server_config::Model,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Notifier<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(req
            .guard::<Connection<'_, Db>>()
            .await
            .map(|conn| conn.into_inner())
            .map_error(|(status, err)| {
                (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr))
            }));
        let notifier = try_outcome!(req
            .guard::<&State<service::dependencies::Notifier>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason:
                        "Could not get a `&State<service::dependencies::Notifier>` from a request."
                            .to_string(),
                }
            )));

        let jid = try_outcome!(JIDGuard::from_request(req).await);
        match Query::is_admin(db, &jid).await {
            Ok(true) => {}
            Ok(false) => {
                debug!("<{}> is not an admin", jid.to_string());
                return Error::Unauthorized.into();
            }
            Err(e) => return Error::DbErr(e).into(),
        }

        match Query::server_config(db).await {
            Ok(Some(server_config)) => Outcome::Success(Notifier {
                db,
                server_config,
                notifier,
            }),
            Ok(None) => Error::PodNotInitialized.into(),
            Err(err) => Error::DbErr(err).into(),
        }
    }
}

impl<'r> Notifier<'r> {
    async fn send(&self, notification: &Notification) -> Result<(), NotifierError> {
        // Store in DB
        let mut model = notification::ActiveModel::new();
        model.created_at = Set(Utc::now());
        model.set_content(notification);
        model.insert(self.db).await?;

        // Try sending
        self.notifier
            .dispatch(notification)
            .map_err(|e| NotifierError::Custom { reason: e })?;

        // Store status if undelivered

        // Delete if delivered

        Ok(())
    }

    pub async fn send_workspace_invitation(
        &self,
        accept_token: Uuid,
        reject_token: Uuid,
    ) -> Result<(), NotifierError> {
        let admin_site_root = PathBuf::from_str(&APP_CONF.branding.page_url.to_string()).unwrap();
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

#[derive(Debug)]
pub enum NotifierError {
    DbErr(DbErr),
    Custom { reason: String },
}

impl fmt::Display for NotifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DbErr(err) => write!(f, "Database error: {err}"),
            Self::Custom { reason } => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for NotifierError {}

impl From<DbErr> for NotifierError {
    fn from(value: DbErr) -> Self {
        Self::DbErr(value)
    }
}
