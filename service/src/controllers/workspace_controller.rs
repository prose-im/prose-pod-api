// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{DatabaseConnection, DbErr, IntoActiveModel as _};

use crate::{
    model::Workspace,
    repositories::WorkspaceRepository,
    sea_orm::{ActiveModelTrait as _, Set},
};

pub struct WorkspaceController<'r> {
    pub db: &'r DatabaseConnection,
}

impl<'r> WorkspaceController<'r> {
    pub async fn get_workspace(&self) -> Result<Workspace, Error> {
        WorkspaceRepository::get(self.db)
            .await?
            .ok_or(Error::WorkspaceNotInitialized)
    }
}

impl<'r> WorkspaceController<'r> {
    pub async fn get_workspace_name(&self) -> Result<String, Error> {
        Ok(self.get_workspace().await?.name)
    }
}

impl<'r> WorkspaceController<'r> {
    pub async fn set_workspace_name(&self, name: String) -> Result<String, Error> {
        let workspace = self.get_workspace().await?;

        let mut active = workspace.into_active_model();
        active.name = Set(name);
        let workspace = active.update(self.db).await?;

        Ok(workspace.name)
    }
}

impl<'r> WorkspaceController<'r> {
    pub async fn get_workspace_icon(&self) -> Result<Option<String>, Error> {
        Ok(self.get_workspace().await?.icon_url)
    }
}

impl<'r> WorkspaceController<'r> {
    pub async fn set_workspace_icon_string(&self, string: String) -> Result<Option<String>, Error> {
        let workspace = self.get_workspace().await?;

        let mut active = workspace.into_active_model();
        // TODO: Validate `string`
        active.icon_url = Set(Some(string));
        let workspace = active.update(self.db).await?;

        Ok(workspace.icon_url)
    }
}

#[derive(Debug)]
pub struct GetWorkspaceAccentColorResponse {
    pub color: Option<String>,
}

impl<'r> WorkspaceController<'r> {
    pub async fn get_workspace_accent_color(&self) -> Result<Option<String>, Error> {
        Ok(self.get_workspace().await?.accent_color)
    }
}

impl<'r> WorkspaceController<'r> {
    pub async fn set_workspace_accent_color(&self, color: String) -> Result<Option<String>, Error> {
        let workspace = self.get_workspace().await?;

        let mut active = workspace.into_active_model();
        // TODO: Validate `color`
        active.accent_color = Set(Some(color));
        let workspace = active.update(self.db).await?;

        Ok(workspace.accent_color)
    }
}

pub type Error = WorkspaceControllerError;

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceControllerError {
    #[error("Workspace not initialized.")]
    WorkspaceNotInitialized,
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),
}
