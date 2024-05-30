// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::models::Member;
use chrono::{DateTime, Utc};
use rocket::{get, put};
use sea_orm_rocket::Connection;
use service::Query;

use crate::error::Error;
use crate::forms::{Timestamp, JID as JIDUriParam};
use crate::guards::Db;
use crate::responders::Paginated;

#[get("/v1/members?<page_number>&<page_size>&<until>")]
pub(super) async fn get_members(
    conn: Connection<'_, Db>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<Paginated<Member>, Error> {
    let db = conn.into_inner();
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };
    let (pages_metadata, members) = Query::get_members(db, page_number, page_size, until).await?;
    Ok(Paginated::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}

/// Get information about one member.
#[get("/v1/members/<jid>")]
pub(super) fn get_member(jid: JIDUriParam) -> String {
    let _jid = jid;
    todo!()
}

/// Change a member's role.
#[put("/v1/members/<_member_id>/role")]
pub(super) fn set_member_role(_member_id: &str) -> String {
    todo!()
}

/// Change a member's Multi-Factor Authentication (MFA) status.
#[put("/v1/members/<_member_id>/mfa")]
pub(super) fn set_member_mfa(_member_id: &str) -> String {
    todo!()
}

/// Log a member out from all of its devices.
#[put("/v1/members/<_member_id>/logout")]
pub(super) fn logout_member(_member_id: &str) -> String {
    todo!()
}
