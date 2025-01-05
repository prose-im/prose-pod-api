// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::Query;
use chrono::{DateTime, Utc};
use service::members::MemberService;

use crate::{error::Error, forms::Pagination, responders::Paginated};

use super::model::*;

pub async fn get_members_route(
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
