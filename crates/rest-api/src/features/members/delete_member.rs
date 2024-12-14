// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::response::status::NoContent;
use sea_orm_rocket::Connection;
use service::members::MemberService;

use crate::{
    error::Error,
    forms::JID as JIDUriParam,
    guards::{Db, LazyGuard},
};

#[delete("/v1/members/<jid>")]
pub async fn delete_member_route<'r>(
    conn: Connection<'r, Db>,
    jid: JIDUriParam,
    member_service: LazyGuard<MemberService>,
) -> Result<NoContent, Error> {
    let db = conn.into_inner();
    let member_service = member_service.inner?;

    member_service.delete_user(db, &jid).await?;

    Ok(NoContent)
}

pub async fn delete_member_route_axum() {
    todo!()
}
