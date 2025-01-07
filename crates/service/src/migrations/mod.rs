// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use crate::features::{
    invitations::migrations::*, members::migrations::*, pod_config::migrations::*,
    server_config::migrations::*, workspace::migrations::*,
};

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231221_172027_create_server_config::Migration),
            Box::new(m20240220_171150_create_member::Migration),
            Box::new(m20240320_095326_create_workspace_invitation::Migration),
            Box::new(m20240506_080027_create_workspace::Migration),
            Box::new(m20240830_080808_create_pod_config::Migration),
            Box::new(m20241214_134500_add_push_notif_config::Migration),
        ]
    }
}
