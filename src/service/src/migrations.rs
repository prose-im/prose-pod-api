// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use crate::features::{
    global_storage::migrations::*, invitations::migrations::*, members::migrations::*,
};

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240220_171150_create_member::Migration),
            Box::new(m20240320_095326_create_workspace_invitation::Migration),
            Box::new(m20250512_131300_create_kv_store::Migration),
            Box::new(m20250531_231100_add_email_address::Migration),
        ]
    }
}
