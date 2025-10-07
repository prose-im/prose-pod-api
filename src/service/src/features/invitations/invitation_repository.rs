// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use std::sync::Arc;

    pub use async_trait::async_trait;
    pub use chrono::{DateTime, TimeDelta, Utc};

    pub use crate::{
        auth::AuthToken,
        errors::Forbidden,
        invitations::{
            invitation_repository::InvitationRepositoryImpl,
            models::{Invitation, InvitationId, InvitationToken},
        },
        members::MemberRole,
        models::EmailAddress,
        util::{
            either::Either,
            paginate::{paginate_iter, ItemsAndPagesNumber},
        },
        xmpp::jid::{JidNode, NodeRef},
    };

    pub use super::{CreateAccountInvitationCommand, InvitationsStats};
}

pub use self::live_invitation_repository::LiveInvitationRepository;
use self::prelude::*;

#[derive(Debug, Clone)]
pub struct InvitationRepository {
    pub implem: Arc<dyn InvitationRepositoryImpl>,
}

#[async_trait]
pub trait InvitationRepositoryImpl: std::fmt::Debug + Sync + Send {
    async fn create_account_invitation(
        &self,
        command: CreateAccountInvitationCommand,
        auth: &AuthToken,
    ) -> Result<Invitation, Either<Forbidden, anyhow::Error>>;

    async fn list_account_invitations(
        &self,
        auth: &AuthToken,
    ) -> Result<Vec<Invitation>, Either<Forbidden, anyhow::Error>>;

    async fn list_account_invitations_paged(
        &self,
        page_number: usize,
        page_size: usize,
        until: Option<DateTime<Utc>>,
        auth: &AuthToken,
    ) -> Result<(ItemsAndPagesNumber, Vec<Invitation>), Either<Forbidden, anyhow::Error>> {
        let full_list: Vec<Invitation> = self.list_account_invitations(auth).await?;

        // Filter based on date.
        let until = until.unwrap_or_else(Utc::now);
        let invitations = full_list
            .into_iter()
            // NOTE: Results aleady sorted by ascending creation date.
            .take_while(|invitation| invitation.created_at <= until);

        // Paginate.
        let (metadata, invitations) = paginate_iter(invitations, page_number, page_size);

        Ok((metadata, invitations))
    }

    async fn account_invitations_stats(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<InvitationsStats, anyhow::Error>;

    /// Used to prevent duplicates.
    async fn get_account_invitation_by_username(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<Option<Invitation>, Either<Forbidden, anyhow::Error>>;

    /// Used to get details for a single invitation.
    async fn get_account_invitation_by_id(
        &self,
        id: &InvitationId,
        auth: &AuthToken,
    ) -> Result<Option<Invitation>, Either<Forbidden, anyhow::Error>>;

    /// Used to show some information in the UI when accepting an invitation.
    async fn get_account_invitation_by_token(
        &self,
        token: &InvitationToken,
    ) -> Result<Option<Invitation>, anyhow::Error>;

    async fn delete_invitation(
        &self,
        token: InvitationToken,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;
}

#[derive(Debug)]
pub struct CreateAccountInvitationCommand {
    pub username: JidNode,
    pub role: MemberRole,
    pub email_address: EmailAddress,
    pub ttl: Option<TimeDelta>,
}

#[derive(Debug)]
pub struct InvitationsStats {
    pub count: usize,
}

mod live_invitation_repository {
    use serde_json::json;

    use crate::invitations::errors::{InvitationNotFound, InvitationNotFoundForToken};
    use crate::prose_pod_server_api::{self, ProsePodServerApi, ProsePodServerError};
    use crate::prosody::prosody_http_admin_api::{CreateAccountInvitationRequest, InviteInfo};
    use crate::prosody::{AsProsody as _, ProsodyHttpAdminApi, ProsodyInvitesRegisterApi};
    use crate::util::either::Either3;
    use crate::TEAM_GROUP_ID;

    use super::*;

    #[derive(Debug)]
    pub struct LiveInvitationRepository {
        pub server_api: ProsePodServerApi,
        pub admin_api: Arc<ProsodyHttpAdminApi>,
        pub invites_register_api: ProsodyInvitesRegisterApi,
    }

    #[async_trait]
    impl InvitationRepositoryImpl for LiveInvitationRepository {
        #[tracing::instrument(level = "trace", skip_all)]
        async fn create_account_invitation(
            &self,
            CreateAccountInvitationCommand {
                username,
                role,
                email_address,
                ttl,
            }: CreateAccountInvitationCommand,
            auth: &AuthToken,
        ) -> Result<Invitation, Either<Forbidden, anyhow::Error>> {
            if let Some(ref ttl) = ttl {
                assert!(ttl.num_seconds() >= 0);
            }

            let admin_api_request = CreateAccountInvitationRequest {
                username: Some(JidNode::from(username)),
                ttl_secs: ttl.map(|ttl| ttl.num_seconds() as u32),
                groups: Some(vec![TEAM_GROUP_ID.to_owned()]),
                roles: Some(vec![role.as_prosody()]),
                note: None,
                additional_data: json!({
                    "email": email_address,
                }),
            };
            let prosody_invite: InviteInfo = self
                .admin_api
                .create_invite_for_account(admin_api_request, auth)
                .await?;

            Invitation::try_from(prosody_invite).map_err(Either::E2)
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn list_account_invitations(
            &self,
            auth: &AuthToken,
        ) -> Result<Vec<Invitation>, Either<Forbidden, anyhow::Error>> {
            let prosody_invites: Vec<InviteInfo> = self.admin_api.list_invites(auth).await?;

            let mut res: Vec<Invitation> = Vec::with_capacity(prosody_invites.len());
            for invite in prosody_invites {
                let id = invite.id.clone();
                match Invitation::try_from(invite) {
                    Ok(invitation) => res.push(invitation),
                    Err(err) => tracing::warn!("Bad invitation {id:?}: {err}"),
                }
            }

            Ok(res)
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn account_invitations_stats(
            &self,
            auth: Option<&AuthToken>,
        ) -> Result<InvitationsStats, anyhow::Error> {
            match self.server_api.invitations_util_stats(auth).await {
                Ok(response) => Ok(InvitationsStats::from(response)),
                // All of those mean something is wrong.
                Err(err @ ProsePodServerError::Unavailable)
                | Err(err @ ProsePodServerError::Forbidden(_))
                | Err(err @ ProsePodServerError::Internal(_)) => Err(anyhow::Error::new(err)),
            }
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn get_account_invitation_by_username(
            &self,
            username: &NodeRef,
            auth: &AuthToken,
        ) -> Result<Option<Invitation>, Either<Forbidden, anyhow::Error>> {
            let prosody_invites = self.admin_api.list_invites(auth).await?;

            match prosody_invites
                .into_iter()
                .find(|invite| invite.jid.node() == Some(username))
            {
                Some(prosody_invite) => match Invitation::try_from(prosody_invite) {
                    Ok(invitation) => Ok(Some(invitation)),
                    Err(err) => Err(Either::E2(err)),
                },
                None => Ok(None),
            }
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn get_account_invitation_by_id(
            &self,
            id: &InvitationId,
            auth: &AuthToken,
        ) -> Result<Option<Invitation>, Either<Forbidden, anyhow::Error>> {
            match self.admin_api.get_invite_by_id(id, auth).await {
                Ok(prosody_invite) => match Invitation::try_from(prosody_invite) {
                    Ok(invitation) => Ok(Some(invitation)),
                    Err(err) => Err(Either::E2(err)),
                },
                Err(Either3::E1(err @ Forbidden(_))) => Err(Either::E1(err)),
                Err(Either3::E2(InvitationNotFound(_))) => Ok(None),
                Err(Either3::E3(err)) => Err(Either::E2(err)),
            }
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn get_account_invitation_by_token(
            &self,
            token: &InvitationToken,
        ) -> Result<Option<Invitation>, anyhow::Error> {
            match self.invites_register_api.get_invite_info(token).await {
                Ok(prosody_invite) => Invitation::try_from(prosody_invite).map(Some),
                Err(Either::E1(InvitationNotFoundForToken)) => Ok(None),
                Err(Either::E2(err)) => Err(err),
            }
        }

        #[tracing::instrument(level = "trace", skip_all)]
        async fn delete_invitation(
            &self,
            token: InvitationToken,
            auth: &AuthToken,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            self.admin_api.delete_invite(&token, auth).await
        }
    }

    impl From<prose_pod_server_api::GetInvitationsStatsResponse> for InvitationsStats {
        fn from(response: prose_pod_server_api::GetInvitationsStatsResponse) -> Self {
            Self {
                count: response.count,
            }
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for InvitationRepository {
    type Target = Arc<dyn InvitationRepositoryImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
