// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, ItemsAndPagesNumber};
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, warn};

use crate::xmpp::{BareJid, ServerCtl, ServerCtlError, XmppService};

use super::{Member, MemberRepository, MemberRole};

#[derive(Debug, Clone)]
pub struct MemberService {
    db: Arc<DatabaseConnection>,
    server_ctl: Arc<ServerCtl>,
    xmpp_service: Arc<XmppService>,
    pub cancellation_token: CancellationToken,
}

impl MemberService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        server_ctl: Arc<ServerCtl>,
        xmpp_service: Arc<XmppService>,
    ) -> Self {
        Self {
            db,
            server_ctl,
            xmpp_service,
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn cancel_tasks(&self) {
        self.cancellation_token.cancel();
    }
}

impl MemberService {
    pub async fn delete_user(
        &self,
        db: &impl ConnectionTrait,
        jid: &BareJid,
    ) -> Result<(), UserDeleteError> {
        // Delete the user from database.
        MemberRepository::delete(db, jid).await?;

        // NOTE: We can't rollback changes made to the XMPP server so we do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.clone();

        // Delete the user from the XMPP server.
        server_ctl
            .remove_user(jid)
            .await
            .map_err(UserDeleteError::XmppServerCannotDeleteUser)?;

        Ok(())
    }

    pub async fn get_members(
        &self,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Member>), DbErr> {
        MemberRepository::get_all(self.db.as_ref(), page_number, page_size, until).await
    }

    pub async fn enrich_member(&self, jid: &BareJid) -> Result<Option<EnrichedMember>, DbErr> {
        trace!("Enriching `{jid}`…");

        let mut member = match MemberRepository::get(self.db.as_ref(), jid).await {
            Ok(Some(entity)) => EnrichedMember {
                jid: jid.to_owned(),
                role: entity.role,
                nickname: None,
                avatar: None,
                online: None,
            },
            Ok(None) => {
                warn!("Member '{jid}' does not exist in database. Won't try enriching it with XMPP data.");
                return Ok(None);
            }
            Err(err) => {
                warn!("Couldn't find member '{jid}' in database: {err}");
                return Err(err);
            }
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
            return Ok(Some(member));
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
            return Ok(Some(member));
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

        Ok(Some(member))
    }
}

#[derive(Debug)]
pub struct EnrichedMember {
    pub jid: BareJid,
    pub role: MemberRole,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum UserDeleteError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("XMPP server cannot delete user: {0}")]
    XmppServerCannotDeleteUser(ServerCtlError),
}
