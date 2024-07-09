// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::{outcome::try_outcome, request::Outcome, Request, State};
use service::{repositories::MemberRepository, services::notifier::Notifier};

use crate::error::{self, Error};
use crate::guards;

use super::{database_connection, LazyFromRequest};

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for Notifier<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);
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

        let jid = try_outcome!(guards::JID::from_request(req).await);
        match MemberRepository::is_admin(db, &jid).await {
            Ok(true) => {}
            Ok(false) => {
                debug!("<{}> is not an admin", jid.to_string());
                return Error::Unauthorized.into();
            }
            Err(e) => return Error::DbErr(e).into(),
        }

        let config = try_outcome!(req
            .guard::<&State<service::config::Config>>()
            .await
            .map_error(|(status, _)| (
                status,
                Error::InternalServerError {
                    reason: "Could not get a `&State<service::config::Config>` from a request."
                        .to_string(),
                }
            )));

        Outcome::Success(Self::new(db, notifier, &config.branding))
    }
}
