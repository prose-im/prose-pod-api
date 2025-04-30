// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{cmp::min, collections::HashSet, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, ItemsAndPagesNumber, Iterable};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, instrument, trace, trace_span, warn, Instrument};

use crate::{
    util::{unaccent, Cache, ConcurrentTaskRunner},
    xmpp::{xmpp_service::Avatar, BareJid, ServerCtl, ServerCtlError, XmppService},
};

use super::{Member, MemberRepository, MemberRole};

#[derive(Debug, Clone)]
struct VCardData {
    pub nickname: Option<String>,
}

lazy_static! {
    /// When enriching members, we query the XMPP server for all vCards. To
    /// avoid flooding the server with too many requests, we cache enriched
    /// members for a little while (enough for someone to finish searching for a
    /// member, but short enough to react to changes). Enriching isn’t a very
    /// costly operation but we wouldn’t want to enrich all members for every
    /// keystroke in the search bar of the Dashboard.
    static ref CACHE_TTL: Duration = Duration::from_secs(2 * 60);

    static ref VCARDS_DATA_CACHE: Cache<BareJid, Option<VCardData>> = Cache::new(*CACHE_TTL);
    static ref AVATARS_CACHE: Cache<BareJid, Option<Avatar>> = Cache::new(*CACHE_TTL);
    static ref ONLINE_STATUSES_CACHE: Cache<BareJid, Option<bool>> = Cache::new(*CACHE_TTL);
}

#[derive(Debug, Clone)]
pub struct MemberService {
    db: Arc<DatabaseConnection>,
    server_ctl: Arc<ServerCtl>,
    xmpp_service: Arc<XmppService>,
    pub cancellation_token: CancellationToken,
    /// A runner used when doing multiple enrichings in parallel.
    concurrent_task_runner: ConcurrentTaskRunner,
    ctx: MemberServiceContext,
}

#[derive(Debug, Clone)]
pub struct MemberServiceContext {
    pub bare_jid: BareJid,
}

impl MemberService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        server_ctl: Arc<ServerCtl>,
        xmpp_service: Arc<XmppService>,
        concurrent_task_runner: ConcurrentTaskRunner,
        ctx: MemberServiceContext,
    ) -> Self {
        Self {
            db,
            server_ctl,
            xmpp_service,
            // NOTE: We reuse the runner’s cancellation token so the runner’s
            //   tasks are also cancelled when one cancels the service’s
            //   `cancellation_token`.
            cancellation_token: concurrent_task_runner.cancellation_token.clone(),
            concurrent_task_runner,
            ctx,
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
        if *jid == self.ctx.bare_jid {
            return Err(UserDeleteError::CannotSelfRemove);
        }

        // Delete the user from database.
        MemberRepository::delete(db, jid).await?;

        // NOTE: We can't rollback changes made to the XMPP server so we do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.clone();

        // Remove the user from everyone's rosters.
        server_ctl
            .remove_team_member(jid)
            .await
            .map_err(UserDeleteError::XmppServerCannotRemoveTeamMember)?;

        // Delete the user from the XMPP server.
        server_ctl
            .remove_user(jid)
            .await
            .map_err(UserDeleteError::XmppServerCannotDeleteUser)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserDeleteError {
    #[error("Cannot self-remove.")]
    CannotSelfRemove,
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("XMPP server cannot remove team member: {0}")]
    XmppServerCannotRemoveTeamMember(ServerCtlError),
    #[error("XMPP server cannot delete user: {0}")]
    XmppServerCannotDeleteUser(ServerCtlError),
}

impl MemberService {
    pub async fn get_members(
        &self,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<Member>), DbErr> {
        MemberRepository::get_page(self.db.as_ref(), page_number, page_size, until).await
    }

    pub async fn search_members(
        &self,
        query: String,
        page_number: u64,
        page_size: u64,
        until: Option<DateTime<Utc>>,
    ) -> Result<(ItemsAndPagesNumber, Vec<EnrichedMember>), DbErr> {
        // Get **all** members from database (no details).
        let members = MemberRepository::get_all_until(self.db.as_ref(), until).await?;

        // Enrich members with vCard data (no avatar nor online status).
        let members = self
            .enrich_members(
                members.into_iter().map(EnrichedMember::from).collect(),
                [EnrichingStep::VCard].into_iter().collect(),
            )
            .await;

        // Filter members based on search query.
        let filtered_members = filter(members, query);

        // Compute pagination metadata.
        let member_count = filtered_members.len() as u64;
        let number_of_pages = member_count.div_ceil(page_size);
        let pages_metadata = ItemsAndPagesNumber {
            number_of_items: member_count,
            number_of_pages,
        };

        // Get only the desired page from the filtered members.
        let start = (page_number - 1) * page_size;
        let end = min(start + page_size, member_count);
        let Some(filtered_members) = filtered_members.get((start as usize)..(end as usize)) else {
            error!("`{start}..{end}` is out of bounds for filtered members.");
            return Ok((pages_metadata, filtered_members));
        };

        // Re-enrich remaining members with missing data.
        let enriched_members = self
            .enrich_members(
                filtered_members.to_vec(),
                [
                    EnrichingStep::Avatar,
                    EnrichingStep::OnlineStatus,
                ]
                .into_iter()
                .collect(),
            )
            .await;

        Ok((pages_metadata, enriched_members))
    }
}

/// Filter members on as much data as possible.
///
/// NOTE: Written as a standalone function so we can test it in the future if we
///   want to.
fn filter(members: Vec<EnrichedMember>, query: String) -> Vec<EnrichedMember> {
    // Normalize the query string (lowercase and remove diacritics).
    let query = unaccent(query).to_lowercase();
    // Get tokens from the query (to match out of order too).
    let query = query.split_whitespace();

    // Filter members on as much data as possible.
    members
        .into_iter()
        .filter(move |member| {
            if member
                .nickname
                .as_ref()
                .is_some_and(|txt| query.clone().any(|s| txt.contains(s)))
            {
                return true;
            }

            if member
                .jid
                .node()
                .is_some_and(|txt| query.clone().any(|s| txt.contains(s)))
            {
                return true;
            }

            false
        })
        .collect()
}

impl MemberService {
    /// Updates a member’s role in database and on the server.
    ///
    /// Returns `None` if the role hasn’t changed.
    ///
    /// Returns the **old** value if the role has changed.
    pub async fn set_member_role(
        &self,
        jid: &BareJid,
        role: MemberRole,
    ) -> Result<Option<MemberRole>, SetMemberRoleError> {
        let new_role =
            (MemberRepository::set_role(self.db.as_ref(), jid, role).await?).replace(role);

        // NOTE: We can't rollback changes made to the XMPP server so we do it
        //   after "rollbackable" DB changes in case they fail. It's not perfect
        //   but better than nothing.
        // TODO: Find a way to rollback XMPP server changes.
        let server_ctl = self.server_ctl.clone();

        server_ctl.set_user_role(jid, &role).await?;

        Ok(new_role)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SetMemberRoleError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
    #[error("XMPP server cannot set user role: {0}")]
    XmppServerCannotSetUserRole(#[from] ServerCtlError),
}

impl MemberService {
    #[instrument(level = "trace", skip_all, fields(jid = jid.to_string()), err)]
    pub async fn enrich_jid(&self, jid: &BareJid) -> Result<Option<EnrichedMember>, DbErr> {
        trace!("Enriching `{jid}`…");

        let member = match MemberRepository::get(self.db.as_ref(), jid).await {
            Ok(Some(entity)) => entity,
            Ok(None) => {
                warn!("Member '{jid}' does not exist in database. Won't try enriching it with XMPP data.");
                return Ok(None);
            }
            Err(err) => {
                warn!("Couldn't find member '{jid}' in database: {err}");
                return Err(err);
            }
        };

        let member = self
            .enrich_member(member.into(), EnrichingStep::iter().collect())
            .await;

        Ok(Some(member))
    }

    #[instrument(
        name = "member_service::enrich_member",
        level = "trace",
        skip_all, fields(jid = member.jid.to_string(), steps)
    )]
    async fn enrich_member(
        &self,
        mut member: EnrichedMember,
        steps: HashSet<EnrichingStep>,
    ) -> EnrichedMember {
        let jid = &member.jid;

        for step in steps {
            if self.cancellation_token.is_cancelled() {
                return member;
            }
            match step {
                EnrichingStep::VCard => {
                    if member.nickname.is_some() {
                        continue;
                    }
                    let (vcard, _) = VCARDS_DATA_CACHE
                        .get_or_insert_with(&member.jid, async || {
                            trace!("Getting `{jid}`'s vCard…");
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
                            vcard.map(|vcard| {
                                let nickname = vcard.nickname.first().cloned().map(|p| p.value);
                                VCardData { nickname }
                            })
                        })
                        .instrument(trace_span!("get_vcard"))
                        .await;
                    if let Some(vcard) = vcard {
                        member.nickname = vcard.nickname;
                    }
                }
                EnrichingStep::Avatar => {
                    if member.avatar.is_some() {
                        continue;
                    }
                    let (avatar, _) = AVATARS_CACHE
                        .get_or_insert_with(&member.jid, async || {
                            trace!("Getting `{jid}`'s avatar…");
                            match self.xmpp_service.get_avatar(jid).await {
                                Ok(Some(avatar)) => Some(avatar),
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
                            }
                        })
                        .instrument(trace_span!("get_avatar"))
                        .await;
                    member.avatar = avatar;
                }
                EnrichingStep::OnlineStatus => {
                    if member.online.is_some() {
                        continue;
                    }
                    let (online, _) = ONLINE_STATUSES_CACHE
                        .get_or_insert_with(&member.jid, async || {
                            trace!("Checking if `{jid}` is connected…");
                            self.xmpp_service
                                .is_connected(jid)
                                .await
                                // Log error
                                .inspect_err(|err| {
                                    warn!("Could not get `{jid}`'s online status: {err}")
                                })
                                // But dismiss it
                                .ok()
                        })
                        .instrument(trace_span!("get_online_status"))
                        .await;
                    member.online = online;
                }
            }
        }

        member
    }

    /// NOTE: Uses cached data automatically.
    async fn enrich_members(
        &self,
        members: Vec<EnrichedMember>,
        steps: HashSet<EnrichingStep>,
    ) -> Vec<EnrichedMember> {
        let mut res = Vec::with_capacity(members.len());

        let member_service = self.clone();
        let mut rx = self.concurrent_task_runner.child().ordered().run(
            members,
            move |member| {
                let member_service = member_service.clone();
                let steps = steps.clone();
                Box::pin(async move { member_service.enrich_member(member, steps).await })
            },
            move || {},
        );
        while let Some(member) = rx.recv().await {
            res.push(member);
        }

        res
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
enum EnrichingStep {
    VCard,
    Avatar,
    OnlineStatus,
}

#[derive(Debug, Clone)]
pub struct EnrichedMember {
    pub jid: BareJid,
    pub role: MemberRole,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub avatar: Option<Avatar>,
}

impl From<Member> for EnrichedMember {
    fn from(member: Member) -> Self {
        Self {
            jid: member.jid(),
            role: member.role,
            online: None,
            nickname: None,
            avatar: None,
        }
    }
}
