pub use sea_orm_migration::prelude::*;

mod m20231221_172027_create_server_config;
mod m20240220_171150_create_member;
mod m20240320_095326_create_workspace_invitation;
mod m20240326_160834_create_notification;
mod m20240506_080027_create_workspace;
mod m20240830_080808_create_pod_config;
mod m20241214_134500_add_push_notif_config;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231221_172027_create_server_config::Migration),
            Box::new(m20240220_171150_create_member::Migration),
            Box::new(m20240320_095326_create_workspace_invitation::Migration),
            Box::new(m20240326_160834_create_notification::Migration),
            Box::new(m20240506_080027_create_workspace::Migration),
            Box::new(m20240830_080808_create_pod_config::Migration),
            Box::new(m20241214_134500_add_push_notif_config::Migration),
        ]
    }
}
