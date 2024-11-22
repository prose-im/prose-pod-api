// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, fmt::Display, future::Future, ops::Deref, sync::Arc};

use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
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
    AppConfig,
};
use tokio::{
    sync::{mpsc, Notify},
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, error};

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

pub async fn run_parallel_tasks<F, R>(
    futures: Vec<F>,
    on_cancel: impl FnOnce() -> (),
    timeout: std::time::Duration,
) -> mpsc::Receiver<R>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let len = futures.len();
    let (tx, mut rx) = mpsc::channel::<R>(len);
    let notify = Arc::new(Notify::new());
    let tasks: FuturesUnordered<JoinHandle<()>> = FuturesUnordered::new();
    for future in futures.into_iter() {
        let tx = tx.clone();
        let notify = notify.clone();
        tasks.push(tokio::spawn(async move {
            let msg = future.await;
            if let Err(err) = tx.send(msg).await {
                if tx.is_closed() {
                    debug!("Cannot send task result: Task aborted.");
                } else {
                    error!("Cannot send task result: {err}");
                }
            }
            notify.notify_waiters();
        }));
    }

    tokio::select! {
        _ = async {
            // NOTE: If `futures.len() == 0` then this `tokio::select!` ends instantly.
            while rx.len() < len {
                // NOTE: Waiting using `rx.recv().await` would consume messages
                //   and we can have only one `Receiver` so we used a `Notify`.
                notify.notified().await
            }
        } => {}
        _ = sleep(timeout) => {
            debug!("Timed out. Cancelling all tasks…");

            rx.close();
            for task in tasks {
                task.abort();
            }
            on_cancel();
        }
    };

    rx
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

    let mut futures = Vec::with_capacity(jids_count);
    for jid in jids.into_iter() {
        let member_controller = member_controller.clone();
        futures.push(async move {
            member_controller
                .enrich_member(&jid)
                .map(EnrichedMember::from)
                .await
        });
    }
    let mut rx = run_parallel_tasks(
        futures,
        || member_controller.cancel_tasks(),
        app_config.default_response_timeout.into_std_duration(),
    )
    .await;

    let mut res = HashMap::with_capacity(jids_count);
    while let Some(member) = rx.recv().await {
        res.insert(member.jid.clone(), member.into());
    }
    Ok(res.into())
}

#[get("/v1/enrich-members?<jids..>", format = "text/event-stream", rank = 2)]
pub fn enrich_members_stream_route<'r>(
    member_controller: LazyGuard<MemberController>,
    jids: Strict<JIDs>,
) -> Result<EventStream![Event + 'r], Error> {
    let member_controller = Arc::new(member_controller.inner?);
    let jids = jids.into_inner();

    Ok(EventStream! {
        fn logged(event: Event) -> Event {
            trace!("Sending {event:?}…");
            event
        }

        let mut tasks: FuturesUnordered<JoinHandle<EnrichedMember>> = FuturesUnordered::new();
        for jid in jids.iter() {
            let jid = jid.clone();
            let member_controller = member_controller.clone();
            tasks.push(tokio::spawn(async move {
                member_controller.enrich_member(&jid).map(EnrichedMember::from).await
            }));
        }

        while let Some(Ok(member)) = tasks.next().await {
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
