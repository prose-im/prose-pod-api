// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, DbErr, ItemsAndPagesNumber};
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, warn};

use crate::xmpp::{BareJid, XmppService};

use super::{Member, MemberRepository};

#[derive(Clone)]
pub struct MemberController {
    pub db: Arc<DatabaseConnection>,
    pub xmpp_service: Arc<XmppService>,
    pub cancellation_token: CancellationToken,
}

impl MemberController {
    pub fn new(db: Arc<DatabaseConnection>, xmpp_service: Arc<XmppService>) -> Self {
        Self {
            db,
            xmpp_service,
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn cancel_tasks(&self) {
        self.cancellation_token.cancel();
    }
}

impl MemberController {
    pub async fn get_members(
        &self,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Member>), DbErr> {
        MemberRepository::get_all(self.db.as_ref(), page_number, page_size, until).await
    }
}

impl MemberController {
    pub async fn enrich_member(&self, jid: &BareJid) -> EnrichedMember {
        trace!("Enriching `{jid}`…");

        let mut member = EnrichedMember {
            jid: jid.to_owned(),
            nickname: None,
            avatar: None,
            online: None,
        };

        trace!("-> Getting `{jid}`'s vCard…");
        let vcard = match self.xmpp_service.get_vcard(jid).await {
            Ok(Some(vcard)) => Some(vcard),
            Ok(None) => {
                debug!("`{jid}` has no vCard.");
                None
            }
            Err(err) => {
                // Log error
                warn!("Could not get `{jid}`'s vCard: {err}");
                // But dismiss it
                None
            }
        };
        member.nickname = vcard
            .and_then(|vcard| vcard.nickname.first().cloned())
            .map(|p| p.value);

        if self.cancellation_token.is_cancelled() {
            return member;
        }
        trace!("-> Getting `{jid}`'s avatar…");
        member.avatar = match self.xmpp_service.get_avatar(jid).await {
            Ok(Some(avatar)) => Some(avatar.base64().to_string()),
            Ok(None) => {
                debug!("`{jid}` has no avatar.");
                None
            }
            Err(err) => {
                // Log error
                warn!("Could not get `{jid}`'s avatar: {err}");
                // But dismiss it
                None
            }
        };

        if self.cancellation_token.is_cancelled() {
            return member;
        }
        trace!("-> Checking if `{jid}` is connected…");
        member.online = self
            .xmpp_service
            .is_connected(jid)
            .await
            // Log error
            .inspect_err(|err| warn!("Could not get `{jid}`'s online status: {err}"))
            // But dismiss it
            .ok();

        member
    }
}

#[derive(Debug)]
pub struct EnrichedMember {
    pub jid: BareJid,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}
