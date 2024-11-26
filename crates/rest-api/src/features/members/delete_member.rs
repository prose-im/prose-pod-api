// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::response::status::NoContent;
use sea_orm_rocket::Connection;
use service::members::UserService;

use crate::{
    error::Error,
    forms::JID as JIDUriParam,
    guards::{Db, LazyGuard},
};

#[delete("/v1/members/<jid>")]
pub async fn delete_member_route<'r>(
    conn: Connection<'r, Db>,
    jid: JIDUriParam,
    user_service: LazyGuard<UserService>,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let user_service = user_service.inner?;

    user_service.delete_user(db, &jid).await?;

    Ok(NoContent)
}
