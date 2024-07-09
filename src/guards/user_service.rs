// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::outcome::try_outcome;
use rocket::request::Outcome;
use rocket::Request;
use service::repositories::MemberRepository;
use service::services::user_service::UserService;

use crate::error::{self, Error};

use super::{database_connection, LazyFromRequest, UnauthenticatedUserService};
use crate::guards;

#[rocket::async_trait]
impl<'r> LazyFromRequest<'r> for UserService<'r> {
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = try_outcome!(database_connection(req).await);

        let jid = try_outcome!(guards::JID::from_request(req).await);
        match MemberRepository::is_admin(db, &jid).await {
            Ok(true) => {}
            Ok(false) => {
                debug!("<{}> is not an admin", jid.to_string());
                return Error::Unauthorized.into();
            }
            Err(e) => return Error::DbErr(e).into(),
        }

        UnauthenticatedUserService::from_request(req)
            .await
            .map(|s| s.0)
    }
}
