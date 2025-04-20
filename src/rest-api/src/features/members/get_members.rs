// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::extract::Query;
use axum_extra::either::Either;
use chrono::{DateTime, Utc};
use service::members::MemberService;

use crate::{
    error::Error,
    forms::{OptionalQuery, Pagination, SearchQuery},
    responders::Paginated,
};

use super::{model::*, EnrichedMember};

pub async fn get_members_route(
    member_service: MemberService,
    Query(pagination): Query<Pagination>,
    search_query: Option<OptionalQuery<SearchQuery>>,
) -> Result<Either<Paginated<Member>, Paginated<EnrichedMember>>, Error> {
    match search_query {
        Some(OptionalQuery(SearchQuery { q: query })) => {
            get_members_filtered(member_service, query, pagination)
                .await
                .map(Either::E2)
        }
        None => get_members(member_service, pagination)
            .await
            .map(Either::E1),
    }
}

pub struct PaginationWithDefaults {
    pub page_number: u64,
    pub page_size: u64,
    pub until: Option<DateTime<Utc>>,
}

fn pagination_with_defaults(
    Pagination {
        page_number,
        page_size,
        until,
    }: Pagination,
) -> Result<PaginationWithDefaults, Error> {
    Ok(PaginationWithDefaults {
        page_number: page_number.unwrap_or(1),
        page_size: page_size.unwrap_or(20),
        until: match until {
            Some(t) => Some(t.try_into()?),
            None => None,
        },
    })
}

async fn get_members(
    member_service: MemberService,
    pagination: Pagination,
) -> Result<Paginated<Member>, Error> {
    let PaginationWithDefaults {
        page_number,
        page_size,
        until,
    } = pagination_with_defaults(pagination)?;

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

async fn get_members_filtered(
    member_service: MemberService,
    query: String,
    pagination: Pagination,
) -> Result<Paginated<EnrichedMember>, Error> {
    let PaginationWithDefaults {
        page_number,
        page_size,
        until,
    } = pagination_with_defaults(pagination)?;

    let (pages_metadata, members) = member_service
        .search_members(query, page_number, page_size, until)
        .await?;

    Ok(Paginated::new(
        members.into_iter().map(Into::into).collect(),
        page_number,
        page_size,
        pages_metadata,
    ))
}
