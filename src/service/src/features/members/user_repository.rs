// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use async_trait::async_trait;

    pub use crate::{
        auth::AuthToken,
        errors::{Forbidden, Unauthorized},
        members::{
            errors::MemberNotFound,
            models::{Member, MemberRole, UsersStats},
            UserRepositoryImpl,
        },
        util::either::{Either, Either3},
        xmpp::jid::NodeRef,
    };
}

use std::sync::Arc;

pub use self::live_user_repository::LiveUserRepository;
use self::prelude::*;

#[derive(Debug, Clone)]
pub struct UserRepository {
    pub implem: Arc<dyn UserRepositoryImpl>,
}

#[async_trait]
pub trait UserRepositoryImpl: std::fmt::Debug + Sync + Send {
    async fn list_users(
        &self,
        auth: &AuthToken,
    ) -> Result<Vec<Member>, Either3<Unauthorized, Forbidden, anyhow::Error>>;

    async fn get_user_by_username(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<Option<Member>, Either<Forbidden, anyhow::Error>>;

    async fn user_exists(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<bool, Either<Forbidden, anyhow::Error>> {
        self.get_user_by_username(username, auth)
            .await
            .map(|opt| opt.is_some())
    }

    async fn users_stats(&self, auth: Option<&AuthToken>) -> Result<UsersStats, anyhow::Error>;

    async fn set_user_role(
        &self,
        username: &NodeRef,
        role: &MemberRole,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;

    async fn delete_user(
        &self,
        username: &NodeRef,
        auth: &AuthToken,
    ) -> Result<(), Either3<MemberNotFound, Forbidden, anyhow::Error>>;
}

#[derive(Debug)]
pub struct UsersStats {
    pub count: usize,
}

mod live_user_repository {
    use crate::{
        auth::AuthService,
        prose_pod_server_api::{self, ProsePodServerApi},
        prosody::{ProsodyAdminRest, ProsodyHttpAdminApi, ProsodyRoleName},
        util::either::to_either3_1_3,
    };

    use super::*;

    #[derive(Debug)]
    pub struct LiveUserRepository {
        pub server_api: ProsePodServerApi,
        pub admin_rest: Arc<ProsodyAdminRest>,
        pub admin_api: Arc<ProsodyHttpAdminApi>,
        pub auth_service: AuthService,
    }

    #[async_trait]
    impl UserRepositoryImpl for LiveUserRepository {
        async fn list_users(
            &self,
            auth: &AuthToken,
        ) -> Result<Vec<Member>, Either3<Unauthorized, Forbidden, anyhow::Error>> {
            // NOTE: This makes one more API call to the Prose Pod Server,
            //   but it’s not a problem yet. If it becomes one, we’ll add
            //   a caching layer.
            let caller = self
                .auth_service
                .get_user_info(auth)
                .await
                .map_err(to_either3_1_3)?;

            // Admins need to see everyone (in roster or not), which means we
            // have to use a dedicated API. However it’s authenticated not to
            // leak sensitive information therefore non-admins would get 403s.
            // As a fallback, we show roster contacts.
            if caller.is_admin() {
                let user_infos = self.admin_api.list_users(auth).await?;

                // Filter out service accounts (using role "prosody:registered"
                // at the moment).
                // TODO: Allow listing service accounts via another route,
                //   or a query param.
                let user_infos = user_infos.into_iter().filter(|info| {
                    if let Some(ref role) = info.role {
                        role.as_str() != ProsodyRoleName::REGISTERED_RAW
                    } else {
                        // Also filter out accounts without a role.
                        // It should not even exist so let’s ignore it.
                        false
                    }
                });

                // Sort by JID, ascending. It doesn’t make much sense,
                // but at least it’s coherent across API calls.
                // FIXME: Move this Server-side.
                let mut user_infos = user_infos.collect::<Vec<_>>();
                user_infos.sort_by(|u1, u2| u1.jid.as_str().cmp(u2.jid.as_str()));

                Ok(user_infos.into_iter().map(Member::from).collect())
            } else {
                // See [Non-admins cannot see users · Issue #346 · prose-im/prose-pod-api](https://github.com/prose-im/prose-pod-api/issues/346).
                Ok(vec![])
            }
        }

        async fn get_user_by_username(
            &self,
            username: &NodeRef,
            auth: &AuthToken,
        ) -> Result<Option<Member>, Either<Forbidden, anyhow::Error>> {
            match self.admin_api.get_user_by_name(username, auth).await {
                Ok(user_info) => Ok(user_info.map(Member::from)),
                Err(err) => Err(err),
            }
        }

        async fn users_stats(&self, auth: Option<&AuthToken>) -> Result<UsersStats, anyhow::Error> {
            match self.server_api.users_util_stats(auth).await {
                Ok(response) => Ok(UsersStats::from(response)),
                Err(err) => Err(anyhow::Error::new(err)),
            }
        }

        async fn set_user_role(
            &self,
            username: &NodeRef,
            role: &MemberRole,
            auth: &AuthToken,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            use crate::prosody::prosody_http_admin_api::UpdateUserInfoRequest;
            use crate::prosody::AsProsody as _;

            self.admin_api
                .update_user(
                    username,
                    &UpdateUserInfoRequest {
                        role: Some(role.as_prosody()),
                        ..Default::default()
                    },
                    auth,
                )
                .await?;

            Ok(())
        }

        async fn delete_user(
            &self,
            username: &NodeRef,
            auth: &AuthToken,
        ) -> Result<(), Either3<MemberNotFound, Forbidden, anyhow::Error>> {
            self.admin_api.delete_user(username, auth).await
        }
    }

    impl From<prose_pod_server_api::GetUsersStatsResponse> for UsersStats {
        fn from(stats: prose_pod_server_api::GetUsersStatsResponse) -> Self {
            Self { count: stats.count }
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for UserRepository {
    type Target = Arc<dyn UserRepositoryImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
