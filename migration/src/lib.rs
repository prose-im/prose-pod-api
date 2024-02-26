pub use sea_orm_migration::prelude::*;

mod m20231221_172027_create_settings;
mod m20240220_171150_create_member;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231221_172027_create_settings::Migration),
            Box::new(m20240220_171150_create_member::Migration),
        ]
    }
}
