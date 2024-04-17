// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt, path::PathBuf, str::FromStr as _};

use chrono::Utc;
use entity::{notification, server_config};
use migration::DbErr;
use rocket::{
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};
use sea_orm_rocket::Connection;
use service::{
    notifier::Notification,
    sea_orm::{prelude::*, ActiveModelBehavior, DatabaseConnection, Set},
    Query, APP_CONF,
};

use crate::error::{self, Error};

use super::{Db, JID as JIDGuard};

pub struct Notifier<'r> {
    // NOTE: We have to wrap model in a `Result` instead of sending `Outcome::Error`
    //   because when sending `Outcome::Error((Status::BadRequest, Error::PodNotInitialized))`
    //   [Rocket's built-in catcher] doesn't use `impl Responder for Error` but instead
    //   transforms the response to HTML (no matter the `Accept` header, which is weird)
    //   saying "The request could not be understood by the server due to malformed syntax.".
    //   We can't build our own [error catcher] as it does not have access to the error
    //   sent via `Outcome::Error`.
    //
    //   [Rocket's built-in catcher]: https://rocket.rs/v0.5/guide/requests/#built-in-catcher
    //   [error catcher]: https://rocket.rs/v0.5/guide/requests/#error-catchers
    pub inner: Result<NotifierInner<'r>, Error>,
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
            .guard::<&State<service::notifier::Notifier>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<service::notifier::Notifier>` from a request."
                        .to_string(),
                }
            )));

        let jid = try_outcome!(req.guard::<JIDGuard>().await);
        match Query::is_admin(db, &jid).await {
            Ok(true) => {}
            Ok(false) => {
                // NOTE: Returning `Error::Unauthorized.into()` would cause the error
                //   to be caught by Rocket's built-in error catcher, not logging it
                //   and returning an incorrect user-facing error.
                //   See note on `Notifier.inner`.
                return Outcome::Success(Self {
                    inner: Err(Error::Unauthorized),
                });
            }
            Err(e) => return Error::DbErr(e).into(),
        }

        Outcome::Success(Self {
            inner: match Query::server_config(db).await {
                Ok(Some(server_config)) => Ok(NotifierInner {
                    db,
                    server_config,
                    notifier,
                }),
                Ok(None) => Err(Error::PodNotInitialized),
                Err(err) => Err(Error::DbErr(err)),
            },
        })
    }
}

pub struct NotifierInner<'r> {
    db: &'r DatabaseConnection,
    notifier: &'r State<service::notifier::Notifier>,
    server_config: server_config::Model,
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

impl<'r> NotifierInner<'r> {
    async fn send(&self, notification: &Notification) -> Result<(), NotifierError> {
        // Store in DB
        let mut model = notification::ActiveModel::new();
        model.created_at = Set(Utc::now());
        model.set_content(notification);
        model.insert(self.db).await?;

        // Try sending
        let Some(notify_config) = &APP_CONF.notify else {
            return Err(NotifierError::Custom {
                reason: "Notifier not configured".to_string(),
            });
        };
        self.notifier
            .lock()
            .expect("Notifier lock poisonned")
            .dispatch(notification, notify_config)
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
