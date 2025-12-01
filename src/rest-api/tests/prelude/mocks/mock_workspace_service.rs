// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::workspace::workspace_service::prelude::*;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MockWorkspaceService {
    pub state: Arc<RwLock<MockWorkspaceServiceState>>,
    pub mock_auth_service: Arc<MockAuthService>,
}

#[derive(Debug)]
pub struct MockWorkspaceServiceState {
    pub workspace: Workspace,
}

#[async_trait]
impl WorkspaceServiceImpl for MockWorkspaceService {
    async fn init_workspace(
        &self,
        InitWorkspaceRequest { name, accent_color }: &InitWorkspaceRequest,
    ) -> Result<(), Either<WorkspaceAlreadyInitialized, anyhow::Error>> {
        if self.mock_auth_service.mock_user_repository.user_count() > 0 {
            return Err(Either::E1(WorkspaceAlreadyInitialized));
        }

        let mut state = self.state.write().unwrap();

        state.workspace.name = name.to_owned();
        state.workspace.accent_color = accent_color.to_owned();

        Ok(())
    }

    async fn get_workspace(&self, _auth: Option<&AuthToken>) -> Result<Workspace, anyhow::Error> {
        Ok(self.state.read().unwrap().workspace.clone())
    }

    async fn patch_workspace(
        &self,
        PatchWorkspaceRequest { name, accent_color }: &PatchWorkspaceRequest,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        self.mock_auth_service.check_admin(auth)?;

        let mut state = self.state.write().unwrap();

        if let Some(name) = name {
            state.workspace.name = name.to_owned();
        }
        if let Some(accent_color) = accent_color {
            state.workspace.accent_color = accent_color.to_owned();
        }

        Ok(())
    }

    async fn get_workspace_name(&self, _auth: Option<&AuthToken>) -> Result<String, anyhow::Error> {
        Ok(self.state.read().unwrap().workspace.name.clone())
    }

    async fn set_workspace_name(
        &self,
        name: &str,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        self.mock_auth_service.check_admin(auth)?;

        self.state.write().unwrap().workspace.name = name.to_owned();

        Ok(())
    }

    async fn get_workspace_accent_color(
        &self,
        _auth: Option<&AuthToken>,
    ) -> Result<Option<Color>, anyhow::Error> {
        Ok(self.state.read().unwrap().workspace.accent_color.clone())
    }

    async fn set_workspace_accent_color(
        &self,
        accent_color: &Option<Color>,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        self.mock_auth_service.check_admin(auth)?;

        self.state.write().unwrap().workspace.accent_color = accent_color.clone();

        Ok(())
    }

    async fn get_workspace_icon(
        &self,
        _auth: Option<&AuthToken>,
    ) -> Result<Option<Avatar>, anyhow::Error> {
        Ok(self.state.read().unwrap().workspace.icon.clone())
    }

    async fn set_workspace_icon(
        &self,
        icon: Avatar,
        auth: &AuthToken,
    ) -> Result<(), Either<Forbidden, anyhow::Error>> {
        self.mock_auth_service.check_admin(auth)?;

        self.state.write().unwrap().workspace.icon = Some(icon);

        Ok(())
    }
}
