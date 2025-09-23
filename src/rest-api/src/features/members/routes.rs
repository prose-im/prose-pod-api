// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, convert::Infallible, fmt::Display};

use axum::{
    extract::{Path, State},
    http::{header::ACCEPT, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    Json,
};
use axum_extra::{either::Either, extract::Query};
use futures::Stream;
use mime::TEXT_EVENT_STREAM;
use service::{
    auth::AuthToken,
    members::{member_controller, EnrichedMember, Member, MemberService},
    models::PaginationForm,
    xmpp::BareJid,
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt as _};
use tracing::warn;

use crate::{
    error::Error,
    forms::{OptionalQuery, SearchQuery},
    responders::Paginated,
    util::headers_ext::HeaderValueExt as _,
    AppState,
};

// MARK: Get one

pub async fn get_member_route(
    Path(jid): Path<BareJid>,
    member_service: MemberService,
) -> Result<Json<EnrichedMember>, Error> {
    match member_controller::get_member(&jid, &member_service).await {
        Ok(member) => Ok(Json(member.into())),
        Err(err) => Err(Error::from(err)),
    }
}

// MARK: Delete one

pub async fn delete_member_route(
    State(AppState { db, .. }): State<AppState>,
    Path(jid): Path<BareJid>,
    ref token: AuthToken,
    member_service: MemberService,
) -> Result<StatusCode, Error> {
    member_controller::delete_member(&db.write, &jid, &member_service, token).await?;
    Ok(StatusCode::NO_CONTENT)
}

// MARK: Get many

pub async fn get_members_route(
    member_service: MemberService,
    Query(pagination): Query<PaginationForm>,
    search_query: Option<OptionalQuery<SearchQuery>>,
) -> Result<Either<Paginated<Member>, Paginated<EnrichedMember>>, Error> {
    if let Some(OptionalQuery(SearchQuery { q: query })) = search_query {
        match member_controller::get_members_filtered(&member_service, pagination, &query).await? {
            members => Ok(Either::E2(members.into())),
        }
    } else {
        match member_controller::get_members(&member_service, pagination).await? {
            members => Ok(Either::E1(members.map(Into::into).into())),
        }
    }
}

pub async fn head_members(
    State(AppState { ref db, .. }): State<AppState>,
) -> Result<Paginated<Member>, Error> {
    match member_controller::head_members(&db.read).await? {
        members => Ok(members.map(Into::into).into()),
    }
}

// MARK: Enriching

#[derive(Debug)]
#[derive(serdev::Deserialize)]
pub struct JIDs {
    jids: Vec<BareJid>,
}

impl Display for JIDs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            (self.jids.iter().map(ToString::to_string))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

pub async fn enrich_members_route(
    member_service: MemberService,
    query: Query<JIDs>,
    headers: HeaderMap,
) -> Either<
    Json<HashMap<BareJid, EnrichedMember>>,
    Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error>,
> {
    match headers.get(ACCEPT) {
        Some(ct) if ct.starts_with(TEXT_EVENT_STREAM.essence_str()) => {
            Either::E2(enrich_members_stream_route_(member_service, query).await)
        }
        _ => Either::E1(enrich_members_route_(member_service, query).await),
    }
}

async fn enrich_members_route_(
    member_service: MemberService,
    Query(JIDs { jids }): Query<JIDs>,
) -> Json<HashMap<BareJid, EnrichedMember>> {
    Json(member_controller::enrich_members(member_service, jids).await)
}

async fn enrich_members_stream_route_(
    member_service: MemberService,
    Query(JIDs { jids }): Query<JIDs>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    let rx = member_controller::enrich_members_stream(member_service, jids);

    let sse_rx = ReceiverStream::new(rx).filter_map(|e| match e {
        Ok(Some(member)) => Some(Ok(Event::default()
            .event("enriched-member")
            .id(member.jid.to_string())
            .json_data(EnrichedMember::from(member))
            .unwrap())),
        Ok(None) => Some(Ok(Event::default()
            .event("end")
            .id("end")
            .comment("End of stream"))),
        Err(err) => {
            warn!("Error enriching member: {err}");
            None
        }
    });

    Ok(Sse::new(sse_rx).keep_alive(KeepAlive::default()))
}
