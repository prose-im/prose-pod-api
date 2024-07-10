// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::{DatabaseConnection, DbErr, IntoActiveModel};

use crate::repositories::{Workspace, WorkspaceRepository};
use crate::sea_orm::{ActiveModelTrait as _, Set};

pub enum WorkspaceController {}

impl WorkspaceController {
    pub async fn get_workspace(db: &DatabaseConnection) -> Result<Option<Workspace>, DbErr> {
        WorkspaceRepository::get(db).await
    }
}

impl WorkspaceController {
    pub async fn get_workspace_name(workspace: Workspace) -> String {
        workspace.name
    }
}

impl WorkspaceController {
    pub async fn set_workspace_name(
        db: &DatabaseConnection,
        workspace: Workspace,
        name: String,
    ) -> Result<String, DbErr> {
        let mut active = workspace.into_active_model();
        active.name = Set(name);
        let workspace = active.update(db).await?;

        Ok(workspace.name)
    }
}

impl WorkspaceController {
    pub fn get_workspace_icon(workspace: Workspace) -> Option<String> {
        workspace.icon_url
    }
}

impl WorkspaceController {
    pub async fn set_workspace_icon_string(
        db: &DatabaseConnection,
        workspace: Workspace,
        string: String,
    ) -> Result<Option<String>, DbErr> {
        // TODO: Validate `string`
        let mut active = workspace.into_active_model();
        active.icon_url = Set(Some(string));
        let workspace = active.update(db).await?;

        Ok(workspace.icon_url)
    }
}

impl WorkspaceController {
    pub fn set_workspace_icon_file(_image: Vec<u8>) -> Vec<u8> {
        todo!()
    }
}

impl WorkspaceController {
    pub fn get_workspace_details_card() -> String {
        todo!()
    }
}

impl WorkspaceController {
    pub fn set_workspace_details_card(_vcard: String) {
        todo!()
    }
}

#[derive(Debug)]
pub struct GetWorkspaceAccentColorResponse {
    pub color: Option<String>,
}

impl WorkspaceController {
    pub fn get_workspace_accent_color(workspace: Workspace) -> Option<String> {
        workspace.accent_color
    }
}

impl WorkspaceController {
    pub async fn set_workspace_accent_color(
        db: &DatabaseConnection,
        workspace: Workspace,
        color: String,
    ) -> Result<Option<String>, DbErr> {
        // TODO: Validate `string`
        let mut active = workspace.into_active_model();
        active.accent_color = Set(Some(color));
        let workspace = active.update(db).await?;

        Ok(workspace.accent_color)
    }
}
