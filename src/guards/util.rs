// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::outcome::try_outcome;
use sea_orm_rocket::{
    rocket::{request::Outcome, Request},
    Connection,
};
use service::{repositories::MemberRepository, sea_orm::DatabaseConnection};

use crate::error::Error;
use crate::guards;

use super::{Db, LazyFromRequest as _};

#[macro_export]
macro_rules! request_state {
    ( $req:expr, $t:ty ) => {
        $req.guard::<&rocket::State<$t>>()
            .await
            .map(|s| s.inner())
            .map_error(|(status, _)| {
                (
                    status,
                    crate::error::Error::InternalServerError {
                        reason: format!(
                            "Could not get a `&State<{}>` from a request.",
                            stringify!($t)
                        ),
                    },
                )
            })
    };
}

pub(super) async fn database_connection<'r, 'a>(
    req: &'r Request<'a>,
) -> Outcome<&'r DatabaseConnection, Error> {
    req.guard::<Connection<'_, Db>>()
        .await
        .map(|conn| conn.into_inner())
        .map_error(|(status, err)| (status, err.map(Error::DbErr).unwrap_or(Error::UnknownDbErr)))
}

pub(super) async fn check_caller_is_admin<'r, 'a>(
    req: &'r Request<'a>,
    db: Option<&'r DatabaseConnection>,
) -> Outcome<(), Error> {
    let db = match db {
        Some(db) => db,
        None => try_outcome!(database_connection(req).await),
    };
    let jid = try_outcome!(guards::JID::from_request(req).await);
    match MemberRepository::is_admin(db, &jid).await {
        Ok(true) => Outcome::Success(()),
        Ok(false) => {
            debug!("<{}> is not an admin", jid.to_string());
            return Error::Unauthorized.into();
        }
        Err(e) => return Error::DbErr(e).into(),
    }
}
