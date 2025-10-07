// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use std::{collections::HashSet, sync::Arc};

    pub use anyhow::Context as _;
    pub use async_trait::async_trait;
    pub use sea_orm::Iterable;
    pub use serdev::Serialize;
    pub use tokio_util::sync::CancellationToken;
    pub use tracing::{debug, instrument, trace, trace_span, warn, Instrument as _};

    pub use crate::{
        app_config::MemberEnrichingConfig,
        auth::{AuthService, AuthToken, Password},
        errors::Forbidden,
        init::errors::FirstAccountAlreadyCreated,
        invitations::invitation_service::AcceptAccountInvitationCommand,
        members::{
            errors::UserDeleteError,
            models::{Member, MemberRole},
            UserRepository,
        },
        models::AvatarOwned,
        prose_pod_server_api::CreateAccountResponse,
        util::{either::Either, paginate::*, unaccent, Cache, ConcurrentTaskRunner, JidExt as _},
        xmpp::{
            jid::{BareJid, JidDomain, NodeRef},
            XmppServiceContext,
        },
    };

    pub use super::{UserApplicationServiceImpl, VCardData};
}

use crate::{errors::Unauthorized, util::either::Either3, xmpp::XmppService};

pub use self::live_user_service::LiveUserApplicationService;
use self::prelude::*;

#[derive(Debug, Clone)]
pub struct MemberService {
    user_repository: UserRepository,
    user_application_service: UserApplicationService,
    server_domain: JidDomain,

    xmpp_service: XmppService,
    auth_service: AuthService,
    pub cancellation_token: CancellationToken,
    /// A runner used when doing multiple enrichings in parallel.
    concurrent_task_runner: ConcurrentTaskRunner,
    vcards_data_cache: Arc<Cache<BareJid, Option<VCardData>>>,
    avatars_cache: Arc<Cache<BareJid, Option<AvatarOwned>>>,
    online_statuses_cache: Arc<Cache<BareJid, Option<bool>>>,
}

#[derive(Debug, Clone)]
pub struct VCardData {
    pub nickname: Option<String>,
    pub email: Option<String>,
}

impl MemberService {
    pub fn new(
        user_repository: UserRepository,
        user_application_service: UserApplicationService,
        server_domain: JidDomain,
        xmpp_service: XmppService,
        auth_service: AuthService,
        concurrent_task_runner: Option<ConcurrentTaskRunner>,
        config: &MemberEnrichingConfig,
    ) -> Self {
        let cache_ttl = config.cache_ttl.into_std_duration();
        let concurrent_task_runner = concurrent_task_runner.unwrap_or_default();
        Self {
            user_repository,
            user_application_service,
            server_domain,
            xmpp_service,
            auth_service,
            // NOTE: We reuse the runner’s cancellation token so the runner’s
            //   tasks are also cancelled when one cancels the service’s
            //   `cancellation_token`.
            cancellation_token: concurrent_task_runner.cancellation_token.clone(),
            concurrent_task_runner,
            vcards_data_cache: Arc::new(Cache::new(cache_ttl)),
            avatars_cache: Arc::new(Cache::new(cache_ttl)),
            online_statuses_cache: Arc::new(Cache::new(cache_ttl)),
        }
    }

    pub fn cancel_tasks(&self) {
        self.cancellation_token.cancel();
    }
}

impl MemberService {
    pub async fn create_first_acount(
        &self,
        username: &NodeRef,
        command: &AcceptAccountInvitationCommand,
    ) -> Result<Member, Either<FirstAccountAlreadyCreated, anyhow::Error>> {
        if self.user_repository.users_stats(None).await?.count > 0 {
            return Err(Either::E1(FirstAccountAlreadyCreated));
        }

        let AcceptAccountInvitationCommand {
            nickname,
            password,
            // TODO(#256): Set email address.
            email: _,
        } = command;
        let jid = BareJid::from_parts(Some(username), &self.server_domain);

        self.user_application_service
            .create_first_acount(username, password)
            .await
            .map_err(|err| match err {
                Either::E1(err) => Either::E1(err),
                Either::E2(err) => Either::E2(anyhow::Error::from(err)),
            })?;

        // Log user in.
        // NOTE: We need to log the user in to get a Prosody
        //   authentication token in order to set the user’s vCard.
        let auth_token = (self.auth_service)
            .log_in(&jid, password)
            .await
            .expect("User credentials should work after creating an account");

        // Set nickname (creates the user’s vCard4).
        let ctx = XmppServiceContext {
            bare_jid: jid,
            auth_token: auth_token.clone(),
        };
        // TODO(#256): Set email address.
        self.xmpp_service
            .create_own_vcard(&ctx, nickname, None)
            .await
            .context("Could not create user vCard4")
            .map_err(Either::E2)?;

        let user = self
            .user_repository
            .get_user_by_username(username, &auth_token)
            .await
            .map_err(anyhow::Error::new)
            .context("Could not get own account info")?
            .expect("User account should exist");

        // Revoke token because it will never be used again.
        self.auth_service
            .revoke(ctx.auth_token)
            .await
            .context("Could not revoke temporary auth token")
            .map_err(Either::E2)?;

        Ok(user)
    }

    pub async fn delete_user(
        &self,
        jid: &BareJid,
        auth: &AuthToken,
    ) -> Result<(), UserDeleteError> {
        assert_eq!(jid.domain(), self.server_domain.as_ref());

        let caller_info = self
            .auth_service
            .get_user_info(auth)
            .await
            .map_err(anyhow::Error::new)?;
        if caller_info.jid == *jid {
            return Err(UserDeleteError::CannotSelfRemove);
        }

        let username = jid.expect_username();

        // Delete the user from the XMPP server.
        // NOTE: Will be removed from groups automatically.
        self.user_repository
            .delete_user(username, auth)
            .await
            .context("XMPP server cannot delete user")?;

        Ok(())
    }
}

fn empty_if_forbidden<T: Default>(
    err: Either3<Unauthorized, Forbidden, anyhow::Error>,
) -> Result<T, Either<Unauthorized, anyhow::Error>> {
    match err {
        Either3::E1(err) => Err(Either::E1(err)),
        Either3::E2(Forbidden(_)) => {
            tracing::warn!("Listing members as a non-admin is temporarily disabled. See [Non-admins cannot see users · Issue #346 · prose-im/prose-pod-api](https://github.com/prose-im/prose-pod-api/issues/346). Returning an empty list.");
            Ok(T::default())
        }
        Either3::E3(err) => Err(Either::E2(err)),
    }
}

impl MemberService {
    pub async fn get_members(
        &self,
        page_number: usize,
        page_size: usize,
        auth: &AuthToken,
    ) -> Result<(ItemsAndPagesNumber, Vec<Member>), Either<Unauthorized, anyhow::Error>> {
        let members = self
            .user_repository
            .list_users(auth)
            .await
            .or_else(empty_if_forbidden)?;
        Ok(paginate_vec_to_vec(&members, page_number, page_size))
    }

    pub async fn search_members(
        &self,
        query: &str,
        page_number: usize,
        page_size: usize,
        ctx: &XmppServiceContext,
    ) -> Result<(ItemsAndPagesNumber, Vec<EnrichedMember>), Either<Unauthorized, anyhow::Error>>
    {
        let ref auth = ctx.auth_token;

        // Get **all** members from database (no details).
        let members = self
            .user_repository
            .list_users(auth)
            .await
            .or_else(empty_if_forbidden)?;

        // Enrich members with vCard data (no avatar nor online status).
        let members = self
            .enrich_members(
                members.into_iter().map(EnrichedMember::from).collect(),
                [EnrichingStep::VCard].into_iter().collect(),
                ctx,
            )
            .await;

        // Filter members based on search query.
        let filtered_members = filter(members, query);

        // Get only the desired page from the filtered members.
        let (pages_metadata, filtered_members) =
            paginate_vec(&filtered_members, page_number, page_size);

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
                ctx,
            )
            .await;

        Ok((pages_metadata, enriched_members))
    }
}

/// Filter members on as much data as possible.
///
/// NOTE: Written as a standalone function so we can test it in the future if we
///   want to.
fn filter(members: Vec<EnrichedMember>, query: &str) -> Vec<EnrichedMember> {
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
    #[instrument(level = "trace", skip_all, fields(jid = jid.to_string()), err)]
    pub async fn enrich_jid(
        &self,
        jid: &BareJid,
        ctx: &XmppServiceContext,
    ) -> Result<Option<EnrichedMember>, anyhow::Error> {
        trace!("Enriching `{jid}`…");

        let ref auth = ctx.auth_token;

        let username = jid.expect_username();

        let user_info = match self
            .user_repository
            .get_user_by_username(username, auth)
            .await?
        {
            Some(entity) => entity,
            None => {
                warn!("Member '{jid}' does not exist. Won't try enriching it with more data.");
                return Ok(None);
            }
        };

        let member = self
            .enrich_member(
                EnrichedMember::from(user_info),
                EnrichingStep::iter().collect(),
                ctx,
            )
            .await;

        Ok(Some(member))
    }

    #[instrument(
        name = "member_service::enrich_member",
        level = "trace",
        skip_all, fields(jid = %member.jid, steps)
    )]
    async fn enrich_member(
        &self,
        mut member: EnrichedMember,
        steps: HashSet<EnrichingStep>,
        ctx: &XmppServiceContext,
    ) -> EnrichedMember {
        let jid = &member.jid;

        for step in steps {
            if self.cancellation_token.is_cancelled() {
                return member;
            }
            match step {
                EnrichingStep::VCard => {
                    if member.nickname.is_some() && member.email.is_some() {
                        continue;
                    }
                    if let Some(vcard) = self.get_vcard(jid, ctx).await {
                        member.nickname = vcard.nickname;
                        member.email = vcard.email;
                    }
                }
                EnrichingStep::Avatar => {
                    if member.avatar.is_some() {
                        continue;
                    }
                    let (avatar, _) = (self.avatars_cache)
                        .get_or_insert_with(&member.jid, async || {
                            trace!("Getting `{jid}`'s avatar…");
                            match self.xmpp_service.get_avatar(ctx, jid).await {
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
                    member.avatar = avatar.to_owned();
                }
                EnrichingStep::OnlineStatus => {
                    if member.online.is_some() {
                        continue;
                    }
                    let (online, _) = (self.online_statuses_cache)
                        .get_or_insert_with(&member.jid, async || {
                            trace!("Checking if `{jid}` is connected…");
                            self.xmpp_service
                                .is_connected(ctx, jid)
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

    #[instrument(name = "get_vcard", level = "trace", skip_all, fields(jid = %jid))]
    pub async fn get_vcard(&self, jid: &BareJid, ctx: &XmppServiceContext) -> Option<VCardData> {
        self.vcards_data_cache
            .get_or_insert_with(jid, async || {
                trace!("Getting `{jid}`'s vCard…");
                let vcard = match self.xmpp_service.get_vcard(ctx, jid).await {
                    Ok(Some(vcard)) => Some(vcard),
                    Ok(None) => {
                        tracing::warn!("`{jid}` has no vCard.");
                        None
                    }
                    Err(err) => {
                        // Log error
                        tracing::warn!("Could not get `{jid}`'s vCard: {err}");
                        // But dismiss it
                        None
                    }
                };
                vcard.map(|vcard| {
                    let nickname = vcard.nickname.first().cloned().map(|p| p.value);
                    let email = vcard.email.first().cloned().map(|p| p.value);
                    VCardData { nickname, email }
                })
            })
            .await
            .0
    }

    /// NOTE: Uses cached data automatically.
    async fn enrich_members(
        &self,
        members: Vec<EnrichedMember>,
        steps: HashSet<EnrichingStep>,
        ctx: &XmppServiceContext,
    ) -> Vec<EnrichedMember> {
        let mut res = Vec::with_capacity(members.len());

        let member_service = self.clone();
        let ctx = ctx.clone();
        let mut rx = self.concurrent_task_runner.child().ordered().run(
            members,
            move |member| {
                let member_service = member_service.clone();
                let steps = steps.clone();
                let ctx = ctx.clone();
                Box::pin(async move { member_service.enrich_member(member, steps, &ctx).await })
            },
            move || {},
        );
        while let Some(member) = rx.recv().await {
            res.push(member);
        }

        res
    }
}

/// [`MemberService`] is has domain logic only, but some actions
/// still need to be mockable and don’t belong in [`UserRepository`].
/// This is where those functions go.
#[derive(Debug, Clone)]
pub struct UserApplicationService {
    pub implem: Arc<dyn UserApplicationServiceImpl>,
}

#[async_trait]
pub trait UserApplicationServiceImpl: std::fmt::Debug + Sync + Send {
    async fn create_first_acount(
        &self,
        username: &NodeRef,
        password: &Password,
    ) -> Result<CreateAccountResponse, Either<FirstAccountAlreadyCreated, anyhow::Error>>;
}

mod live_user_service {
    use crate::prose_pod_server_api::ProsePodServerApi;

    use super::*;

    #[derive(Debug)]
    pub struct LiveUserApplicationService {
        pub server_api: ProsePodServerApi,
    }

    #[async_trait]
    impl UserApplicationServiceImpl for LiveUserApplicationService {
        async fn create_first_acount(
            &self,
            username: &NodeRef,
            password: &Password,
        ) -> Result<CreateAccountResponse, Either<FirstAccountAlreadyCreated, anyhow::Error>>
        {
            self.server_api
                .init_first_account(username, password)
                .await
                .map_err(|err| match err {
                    Either::E1(err) => Either::E1(err),
                    Either::E2(err) => Either::E2(anyhow::Error::from(err)),
                })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
enum EnrichingStep {
    VCard,
    Avatar,
    OnlineStatus,
}

#[derive(Debug, Clone)]
#[derive(Serialize)]
pub struct EnrichedMember {
    pub jid: BareJid,
    pub role: MemberRole,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<AvatarOwned>,
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

// MARK: - Boilerplate

impl std::ops::Deref for UserApplicationService {
    type Target = Arc<dyn UserApplicationServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}

impl From<Member> for EnrichedMember {
    fn from(member: Member) -> Self {
        Self {
            jid: member.jid,
            role: member.role,
            online: None,
            nickname: None,
            email: None,
            avatar: None,
        }
    }
}
