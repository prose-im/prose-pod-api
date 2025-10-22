// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use async_trait::async_trait;

    pub use crate::{
        auth::AuthToken,
        errors::Forbidden,
        models::{Avatar, Color},
        prose_pod_server_api::{InitWorkspaceRequest, PatchWorkspaceRequest},
        util::either::Either,
        workspace::{errors::WorkspaceAlreadyInitialized, Workspace},
    };

    pub use super::WorkspaceServiceImpl;
}

use std::sync::Arc;

pub use self::live_workspace_service::LiveWorkspaceService;
use self::prelude::*;

/// [`MemberService`] has domain logic only, but some actions
/// still need to be mockable. This is where those functions go.
#[derive(Debug, Clone)]
pub struct WorkspaceService {
    pub implem: Arc<dyn WorkspaceServiceImpl>,
}

#[async_trait]
pub trait WorkspaceServiceImpl: std::fmt::Debug + Sync + Send {
    async fn init_workspace(
        &self,
        req: &InitWorkspaceRequest,
    ) -> Result<(), Either<WorkspaceAlreadyInitialized, anyhow::Error>>;

    async fn get_workspace(&self, auth: Option<&AuthToken>) -> Result<Workspace, anyhow::Error>;

    async fn patch_workspace(
        &self,
        req: &PatchWorkspaceRequest,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;

    async fn get_workspace_name(&self, auth: Option<&AuthToken>) -> Result<String, anyhow::Error>;

    async fn set_workspace_name(
        &self,
        name: &str,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;

    async fn get_workspace_accent_color(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<Option<Color>, anyhow::Error>;

    async fn set_workspace_accent_color(
        &self,
        accent_color: &Option<Color>,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;

    async fn get_workspace_icon(
        &self,
        auth: Option<&AuthToken>,
    ) -> Result<Option<Avatar>, anyhow::Error>;

    async fn set_workspace_icon(
        &self,
        icon: Avatar,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>>;
}

mod live_workspace_service {
    use crate::{
        prose_pod_server_api::{ProsePodServerApi, ProsePodServerError},
        xmpp::JidNode,
    };

    use super::*;

    #[derive(Debug)]
    pub struct LiveWorkspaceService {
        pub server_api: ProsePodServerApi,
        pub workspace_username: JidNode,
    }

    #[async_trait]
    impl WorkspaceServiceImpl for LiveWorkspaceService {
        async fn init_workspace(
            &self,
            req: &InitWorkspaceRequest,
        ) -> Result<(), Either<WorkspaceAlreadyInitialized, anyhow::Error>> {
            self.server_api.init_workspace(req).await.map_err(erase_e2)
        }

        async fn get_workspace(
            &self,
            auth: Option<&AuthToken>,
        ) -> Result<Workspace, anyhow::Error> {
            match self.server_api.get_workspace(auth).await.into() {
                Ok(response) => Ok(Workspace::from(response)),
                Err(err) => Err(anyhow::Error::from(err)),
            }
        }

        async fn patch_workspace(
            &self,
            req: &PatchWorkspaceRequest,
            auth: &AuthToken,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            self.server_api
                .patch_workspace(req, auth)
                .await
                .map_err(separate_forbidden)
        }

        async fn get_workspace_name(
            &self,
            auth: Option<&AuthToken>,
        ) -> Result<String, anyhow::Error> {
            self.server_api
                .get_workspace_name(auth)
                .await
                .map_err(anyhow::Error::from)
        }

        async fn set_workspace_name(
            &self,
            name: &str,
            auth: &AuthToken,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            self.server_api
                .set_workspace_name(name, auth)
                .await
                .map_err(separate_forbidden)
        }

        async fn get_workspace_accent_color(
            &self,
            auth: Option<&AuthToken>,
        ) -> Result<Option<Color>, anyhow::Error> {
            self.server_api
                .get_workspace_accent_color(auth)
                .await
                .map_err(anyhow::Error::from)
        }

        async fn set_workspace_accent_color(
            &self,
            accent_color: &Option<Color>,
            auth: &AuthToken,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            self.server_api
                .set_workspace_accent_color(accent_color, auth)
                .await
                .map_err(separate_forbidden)
        }

        async fn get_workspace_icon(
            &self,
            auth: Option<&AuthToken>,
        ) -> Result<Option<Avatar>, anyhow::Error> {
            self.server_api
                .get_workspace_icon(auth)
                .await
                .map_err(anyhow::Error::from)
        }

        async fn set_workspace_icon(
            &self,
            icon: Avatar,
            auth: &AuthToken,
        ) -> Result<(), Either<Forbidden, anyhow::Error>> {
            self.server_api
                .set_workspace_icon(icon, auth)
                .await
                .map_err(separate_forbidden)
        }
    }

    // MARK: - Helpers

    fn separate_forbidden(err: ProsePodServerError) -> Either<Forbidden, anyhow::Error> {
        match err {
            ProsePodServerError::Forbidden(err) => Either::E1(err),
            err => Either::E2(anyhow::Error::from(err)),
        }
    }

    fn erase_e2<E>(err: Either<E, ProsePodServerError>) -> Either<E, anyhow::Error> {
        match err {
            Either::E1(err) => Either::E1(err),
            Either::E2(err) => Either::E2(anyhow::Error::from(err)),
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for WorkspaceService {
    type Target = Arc<dyn WorkspaceServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.implem
    }
}
