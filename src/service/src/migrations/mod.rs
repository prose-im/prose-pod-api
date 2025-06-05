// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use crate::features::{
    global_storage::migrations::*, invitations::migrations::*, members::migrations::*,
    pod_config::migrations::*, server_config::migrations::*, workspace::migrations::*,
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
            Box::new(m20250112_122600_create_tls_profile::Migration),
            Box::new(m20250112_184600_add_federation_fields::Migration),
            Box::new(m20250323_222400_delete_workspace::Migration),
            Box::new(m20250326_095800_add_dashboard_address::Migration),
            Box::new(m20250331_222300_dashboard_address_to_url::Migration),
            Box::new(m20250404_193000_add_prosody_overrides::Migration),
            Box::new(m20250407_014800_add_prosody_overrides_raw::Migration),
            Box::new(m20250512_131300_create_kv_store::Migration),
            Box::new(m20250531_231100_add_email_address::Migration),
        ]
    }
}
