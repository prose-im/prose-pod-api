// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, DbErr, ItemsAndPagesNumber};
use tracing::{debug, trace, warn};

use crate::xmpp::{BareJid, XmppService};

use super::{Member, MemberRepository};

pub struct MemberController<'r> {
    pub db: &'r DatabaseConnection,
    pub xmpp_service: XmppService<'r>,
}

impl<'r> MemberController<'r> {
    pub async fn get_members(
        &self,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Member>), DbErr> {
        MemberRepository::get_all(self.db, page_number, page_size, until).await
    }
}

impl<'r> MemberController<'r> {
    pub async fn enrich_member(&self, jid: &BareJid) -> EnrichedMember {
        trace!("Enriching `{jid}`…");

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
        let nickname = vcard
            .and_then(|vcard| vcard.nickname.first().cloned())
            .map(|p| p.value);

        let avatar = match self.xmpp_service.get_avatar(jid).await {
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

        let online = self
            .xmpp_service
            .is_connected(jid)
            .await
            // Log error
            .inspect_err(|err| warn!("Could not get `{jid}`'s online status: {err}"))
            // But dismiss it
            .ok();

        EnrichedMember {
            jid: jid.to_owned(),
            nickname,
            avatar,
            online,
        }
    }
}

#[derive(Debug)]
pub struct EnrichedMember {
    pub jid: BareJid,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}
