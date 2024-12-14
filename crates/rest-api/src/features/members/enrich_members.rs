// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, fmt::Display, ops::Deref, sync::Arc};

use rocket::{
    form::Strict,
    get,
    response::stream::{Event, EventStream},
    serde::json::Json,
    State,
};
use serde::{Deserialize, Serialize};
use service::{
    members::{member_service, MemberRole, MemberService},
    models::BareJid,
    util::ConcurrentTaskRunner,
    AppConfig,
};
use tracing::trace;

use crate::{error::Error, forms::JID as JIDUriParam, guards::LazyGuard};

use super::Member;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedMember {
    pub jid: BareJid,
    pub role: MemberRole,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, FromForm)]
pub struct JIDs {
    jids: Vec<JIDUriParam>,
}

#[get("/v1/enrich-members?<jids..>", format = "application/json")]
pub async fn enrich_members_route(
    member_service: LazyGuard<MemberService>,
    jids: Strict<JIDs>,
    app_config: &State<AppConfig>,
) -> Result<Json<HashMap<BareJid, EnrichedMember>>, Error> {
    let member_service = member_service.inner?;
    let jids = jids.into_inner().jids;
    let jids_count = jids.len();
    let runner = ConcurrentTaskRunner::default(&app_config);

    let cancellation_token = member_service.cancellation_token.clone();
    let mut rx = runner.run(
        jids,
        move |jid| {
            let member_service = member_service.clone();
            Box::pin(async move { member_service.enrich_member(&jid).await })
        },
        move || cancellation_token.cancel(),
    );

    let mut res = HashMap::with_capacity(jids_count);
    while let Some(Ok(Some(member))) = rx.recv().await {
        res.insert(member.jid.clone(), EnrichedMember::from(member).into());
    }
    Ok(res.into())
}

pub async fn enrich_members_route_axum() {
    todo!()
}

#[get("/v1/enrich-members?<jids..>", format = "text/event-stream", rank = 2)]
pub async fn enrich_members_stream_route<'r>(
    member_service: LazyGuard<MemberService>,
    jids: Strict<JIDs>,
    app_config: &State<AppConfig>,
) -> Result<EventStream![Event + 'r], Error> {
    let member_service = Arc::new(member_service.inner?);
    let jids = jids.into_inner().jids;
    let runner = ConcurrentTaskRunner::default(&app_config);

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let cancellation_token = member_service.cancellation_token.clone();
        let mut rx = runner.run(
            jids,
            move |jid| {
                let member_service = member_service.clone();
                Box::pin(async move { member_service .enrich_member(&jid).await })
            },
            move || cancellation_token.cancel(),
        );

        while let Some(Ok(Some(member))) = rx.recv().await {
            let jid = member.jid.clone();
            yield logged(Event::json(&EnrichedMember::from(member)).id(jid.to_string()).event("enriched-member"));
        }

        yield logged(Event::empty().event("end").id("end").with_comment("End of stream"));
    })
}

pub async fn enrich_members_stream_route_axum() {
    todo!()
}

// BOILERPLATE

impl From<member_service::EnrichedMember> for EnrichedMember {
    fn from(value: member_service::EnrichedMember) -> Self {
        Self {
            jid: value.jid,
            role: value.role,
            online: value.online,
            nickname: value.nickname,
            avatar: value.avatar,
        }
    }
}

// NOTE: This is just to ensure that `EnrichedMember` is a supertype of `Member`.
impl From<EnrichedMember> for Member {
    fn from(value: EnrichedMember) -> Self {
        Self {
            jid: value.jid,
            role: value.role,
        }
    }
}

impl Deref for JIDs {
    type Target = Vec<JIDUriParam>;

    fn deref(&self) -> &Self::Target {
        &self.jids
    }
}

impl Display for JIDs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.jids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
