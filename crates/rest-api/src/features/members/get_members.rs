// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::Query;
use chrono::{DateTime, Utc};
use service::{auth::UserInfo, members::MemberService};

use crate::{
    error::Error,
    forms::{Pagination, Timestamp},
    guards::LazyGuard,
    responders::{Paginated, PaginatedRocket},
};

use super::model::*;

#[rocket::get("/v1/members?<page_number>&<page_size>&<until>")]
pub async fn get_members_route<'r>(
    member_service: LazyGuard<MemberService>,
    user_info: LazyGuard<UserInfo>,
    page_number: Option<u64>,
    page_size: Option<u64>,
    until: Option<Timestamp>,
) -> Result<PaginatedRocket<Member>, Error> {
    // Make sure the user is logged in.
    let _ = user_info.inner?;

    let member_service = member_service.inner?;
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };

    let (pages_metadata, members) = member_service
        .get_members(page_number, page_size, until)
        .await?;

    Ok(PaginatedRocket::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}

pub async fn get_members_route_axum(
    member_service: MemberService,
    Query(Pagination {
        page_number,
        page_size,
        until,
    }): Query<Pagination>,
) -> Result<Paginated<Member>, Error> {
    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    let until: Option<DateTime<Utc>> = match until {
        Some(t) => Some(t.try_into()?),
        None => None,
    };

    let (pages_metadata, members) = member_service
        .get_members(page_number, page_size, until)
        .await?;

    Ok(Paginated::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}
