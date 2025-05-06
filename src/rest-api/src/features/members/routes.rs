// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, convert::Infallible, fmt::Display};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    Json,
};
use axum_extra::{either::Either, extract::Query};
use futures::Stream;
use serde::Deserialize;
use service::{
    members::{member_controller, EnrichedMember, Member, MemberService},
    models::PaginationForm,
    xmpp::BareJid,
    AppConfig,
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt as _};
use tracing::warn;

use crate::{
    error::Error,
    forms::{OptionalQuery, SearchQuery},
    responders::Paginated,
    AppState,
};

// GET ONE

pub async fn get_member_route(
    Path(jid): Path<BareJid>,
    member_service: MemberService,
) -> Result<Json<EnrichedMember>, Error> {
    match member_controller::get_member(&jid, &member_service).await {
        Ok(member) => Ok(Json(member.into())),
        Err(err) => Err(Error::from(err)),
    }
}

// DELETE ONE

pub async fn delete_member_route(
    State(AppState { db, .. }): State<AppState>,
    Path(jid): Path<BareJid>,
    member_service: MemberService,
) -> Result<StatusCode, Error> {
    member_controller::delete_member(&db, &jid, &member_service).await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET MANY

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
    match member_controller::head_members(db).await? {
        members => Ok(members.map(Into::into).into()),
    }
}

// ENRICHING

#[derive(Debug, Clone, Deserialize)]
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
    Query(JIDs { jids }): Query<JIDs>,
    app_config: AppConfig,
) -> Json<HashMap<BareJid, EnrichedMember>> {
    Json(member_controller::enrich_members(member_service, jids, &app_config).await)
}

pub async fn enrich_members_stream_route(
    member_service: MemberService,
    Query(JIDs { jids }): Query<JIDs>,
    State(AppState { app_config, .. }): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    let rx = member_controller::enrich_members_stream(member_service, jids, &app_config);

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
