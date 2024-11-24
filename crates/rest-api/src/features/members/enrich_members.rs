// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, fmt::Display, ops::Deref, sync::Arc};

use futures::FutureExt as _;
use rocket::{
    form::Strict,
    get,
    response::stream::{Event, EventStream},
    serde::json::Json,
    State,
};
use serde::{Deserialize, Serialize};
use service::{
    members::{member_controller, MemberController},
    models::BareJid,
    util::ConcurrentTaskRunner,
    AppConfig,
};

use crate::{error::Error, forms::JID as JIDUriParam, guards::LazyGuard};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedMember {
    pub jid: BareJid,
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
    member_controller: LazyGuard<MemberController>,
    jids: Strict<JIDs>,
    app_config: &State<AppConfig>,
) -> Result<Json<HashMap<BareJid, EnrichedMember>>, Error> {
    let member_controller = member_controller.inner?;
    let jids = jids.into_inner().jids;
    let jids_count = jids.len();
    let runner = ConcurrentTaskRunner::default(&app_config);

    let futures = jids
        .into_iter()
        .map(|jid| {
            let member_controller = member_controller.clone();
            Box::pin(async move {
                member_controller
                    .enrich_member(&jid)
                    .map(EnrichedMember::from)
                    .await
            })
        })
        .collect::<Vec<_>>();
    let mut rx = runner.run(futures, move || member_controller.cancel_tasks());

    let mut res = HashMap::with_capacity(jids_count);
    while let Some(member) = rx.recv().await {
        res.insert(member.jid.clone(), member.into());
    }
    Ok(res.into())
}

#[get("/v1/enrich-members?<jids..>", format = "text/event-stream", rank = 2)]
pub async fn enrich_members_stream_route<'r>(
    member_controller: LazyGuard<MemberController>,
    jids: Strict<JIDs>,
    app_config: &State<AppConfig>,
) -> Result<EventStream![Event + 'r], Error> {
    let member_controller = Arc::new(member_controller.inner?);
    let jids = jids.into_inner().jids;
    let runner = ConcurrentTaskRunner::default(&app_config);

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let futures = jids
            .into_iter()
            .map(|jid| {
                let member_controller = member_controller.clone();
                Box::pin(async move {
                    member_controller
                        .enrich_member(&jid)
                        .map(EnrichedMember::from)
                        .await
                })
            })
            .collect::<Vec<_>>();
        let mut rx = runner.run(
            futures,
            move || member_controller.cancel_tasks(),
        );

        while let Some(member) = rx.recv().await {
            let jid = member.jid.clone();
            yield logged(Event::json(&member).id(jid.to_string()).event("enriched-member"));
        }

        yield logged(Event::empty().event("end").id("end").with_comment("End of stream"));
    })
}

// BOILERPLATE

impl From<member_controller::EnrichedMember> for EnrichedMember {
    fn from(value: member_controller::EnrichedMember) -> Self {
        Self {
            jid: value.jid,
            online: value.online,
            nickname: value.nickname,
            avatar: value.avatar,
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
