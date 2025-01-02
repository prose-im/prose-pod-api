// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use rocket::response::status::NoContent;
use sea_orm_rocket::Connection;
use service::{members::MemberService, xmpp::BareJid};

use crate::{
    error::Error,
    forms::JID as JIDUriParam,
    guards::{Db, LazyGuard},
    AppState,
};

#[rocket::delete("/v1/members/<jid>")]
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

pub async fn delete_member_route_axum(
    State(AppState { db, .. }): State<AppState>,
    Path(jid): Path<BareJid>,
    member_service: MemberService,
) -> Result<StatusCode, Error> {
    member_service.delete_user(&db, &jid).await?;
    Ok(StatusCode::NO_CONTENT)
}
