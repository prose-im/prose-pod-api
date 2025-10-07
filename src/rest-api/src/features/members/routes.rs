// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, convert::Infallible, fmt::Display};

use axum::{
    extract::Path,
    http::{header::ACCEPT, HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use axum_extra::{either::Either, extract::Query};
use futures::Stream;
use mime::TEXT_EVENT_STREAM;
use service::{
    auth::AuthToken,
    members::{member_controller, EnrichedMember, Member, MemberService},
    models::PaginationForm,
    xmpp::{BareJid, XmppServiceContext},
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt as _};
use tracing::warn;

use crate::{
    error::Error,
    forms::{OptionalQuery, SearchQuery},
    responders::Paginated,
    util::headers_ext::HeaderValueExt as _,
};

// MARK: Get one

pub async fn get_member_route(
    ref member_service: MemberService,
    ref ctx: XmppServiceContext,
    Path(ref jid): Path<BareJid>,
) -> Result<Json<EnrichedMember>, Error> {
    match member_controller::get_member(jid, member_service, ctx).await {
        Ok(member) => Ok(Json(member.into())),
        Err(err) => Err(Error::from(err)),
    }
}

// MARK: Delete one

pub async fn delete_member_route(
    ref member_service: MemberService,
    ref auth: AuthToken,
    Path(ref jid): Path<BareJid>,
) -> Result<StatusCode, Error> {
    member_controller::delete_member(jid, member_service, auth).await?;
    Ok(StatusCode::NO_CONTENT)
}

// MARK: Get many

pub async fn get_members_route(
    ref member_service: MemberService,
    ref auth: AuthToken,
    ref ctx: XmppServiceContext,
    Query(pagination): Query<PaginationForm>,
    search_query: Option<OptionalQuery<SearchQuery>>,
) -> Result<Either<Paginated<Member>, Paginated<EnrichedMember>>, Error> {
    if let Some(OptionalQuery(SearchQuery { q: ref query })) = search_query {
        match member_controller::get_members_filtered(member_service, pagination, query, ctx)
            .await?
        {
            members => Ok(Either::E2(members.into())),
        }
    } else {
        match member_controller::get_members(member_service, pagination, auth).await? {
            members => Ok(Either::E1(members.map(Into::into).into())),
        }
    }
}

pub async fn head_members(
    ref member_service: MemberService,
    ref auth: AuthToken,
) -> Result<Paginated<Member>, Error> {
    match member_controller::head_members(member_service, auth).await? {
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
    ctx: XmppServiceContext,
    headers: HeaderMap,
    query: Query<JIDs>,
) -> Either<
    Json<HashMap<BareJid, EnrichedMember>>,
    Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error>,
> {
    match headers.get(ACCEPT) {
        Some(ct) if ct.starts_with(TEXT_EVENT_STREAM.essence_str()) => {
            Either::E2(enrich_members_stream_route_(member_service, ctx, query).await)
        }
        _ => Either::E1(enrich_members_route_(member_service, ctx, query).await),
    }
}

async fn enrich_members_route_(
    member_service: MemberService,
    ctx: XmppServiceContext,
    Query(JIDs { jids }): Query<JIDs>,
) -> Json<HashMap<BareJid, EnrichedMember>> {
    Json(member_controller::enrich_members(member_service, jids, ctx).await)
}

async fn enrich_members_stream_route_(
    member_service: MemberService,
    ctx: XmppServiceContext,
    Query(JIDs { jids }): Query<JIDs>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    let rx = member_controller::enrich_members_stream(member_service, jids, ctx);

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
