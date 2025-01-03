// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::outcome::try_outcome;
use sea_orm_rocket::{
    rocket::{request::Outcome, Request},
    Connection,
};
use service::{auth::UserInfo, members::MemberRepository, sea_orm::DatabaseConnection};

use crate::error::{self, Error};

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
                    Error::from(crate::error::InternalServerError(format!(
                        "Could not get a `&State<{}>` from a request.",
                        stringify!($t)
                    ))),
                )
            })
    };
}

pub async fn database_connection<'r, 'a>(
    req: &'r Request<'a>,
) -> Outcome<&'r DatabaseConnection, Error> {
    req.guard::<Connection<'_, Db>>()
        .await
        .map(|conn| conn.into_inner())
        .map_error(|(status, err)| {
            (
                status,
                err.map(Into::into).unwrap_or(error::UnknownDbErr.into()),
            )
        })
}

pub async fn check_caller_is_admin<'r, 'a>(
    req: &'r Request<'a>,
    db: Option<&'r DatabaseConnection>,
) -> Outcome<(), Error> {
    let db = match db {
        Some(db) => db,
        None => try_outcome!(database_connection(req).await),
    };
    let jid = try_outcome!(UserInfo::from_request(req).await).jid;
    match MemberRepository::is_admin(db, &jid).await {
        Ok(true) => Outcome::Success(()),
        Ok(false) => {
            return Error::from(error::Forbidden(format!("<{jid}> is not an admin"))).into();
        }
        Err(e) => return Error::from(e).into(),
    }
}
