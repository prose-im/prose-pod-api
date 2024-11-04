// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, fmt::Display, ops::Deref};

use rocket::{
    form::Strict,
    get,
    response::stream::{Event, EventStream},
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use service::{
    controllers::member_controller::{self, MemberController},
    prose_xmpp::BareJid,
};

use crate::{error::Error, forms::JID as JIDUriParam, guards::LazyGuard};

#[derive(Debug, Serialize, Deserialize)]
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
pub async fn enrich_members_route<'r>(
    member_controller: LazyGuard<MemberController<'r>>,
    jids: Strict<JIDs>,
) -> Result<Json<HashMap<BareJid, EnrichedMember>>, Error> {
    let member_controller = member_controller.inner?;
    let jids = jids.into_inner();

    let mut res = HashMap::with_capacity(jids.len());
    for jid in jids.iter() {
        let enriched_member = member_controller.enrich_member(jid).await;
        res.insert(jid.deref().to_owned(), enriched_member.into());
    }
    Ok(res.into())
}

#[get("/v1/enrich-members?<jids..>", format = "text/event-stream", rank = 2)]
pub fn enrich_members_stream_route<'r>(
    member_controller: LazyGuard<MemberController<'r>>,
    jids: Strict<JIDs>,
) -> Result<EventStream![Event + 'r], Error> {
    let member_controller = member_controller.inner?;
    let jids = jids.into_inner();

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        for jid in jids.iter() {
            let res: EnrichedMember = member_controller.enrich_member(jid).await.into();
            yield logged(Event::json(&res).id(jid.to_string()).event("enriched-member"));
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
