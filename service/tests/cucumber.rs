// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prosody;

use ::entity::server_config::{self, Model as ServerConfig};
use ::migration::{self, MigratorTrait};
use cucumber::World;
use prosody_config::ProsodyConfigFile;
use sea_orm::*;
use service::Mutation;

pub const DEFAULT_WORKSPACE_NAME: &'static str = "Prose";

#[tokio::main]
async fn main() {
    // Run tests and ignore undefined steps
    TestWorld::run("tests/features").await;

    // Run and fail on undefined steps
    // TestWorld::cucumber()
    //     .fail_on_skipped()
    //     .run_and_exit("tests/features").await;
}

#[derive(Debug, World)]
#[world(init = Self::new)]
struct TestWorld {
    db: DatabaseConnection,
    server_config: ServerConfig,
    prosody_config: Option<ProsodyConfigFile>,
}

impl TestWorld {
    async fn new() -> Self {
        // Connecting SQLite
        let db = match Database::connect("sqlite::memory:").await {
            Ok(conn) => conn,
            Err(e) => panic!("Could not connect to test database: {e}"),
        };

        // Setup database schema
        if let Err(e) = migration::Migrator::up(&db, None).await {
            panic!("Could not setup test database schema: {e}");
        }

        let form = server_config::ActiveModel {
            workspace_name: Set(DEFAULT_WORKSPACE_NAME.to_string()),
            ..Default::default()
        };
        let server_config = match Mutation::create_server_config(&db, form).await {
            Ok(conf) => conf,
            Err(e) => panic!("Could not create server config: {e}"),
        };
        let server_config = match server_config.try_into_model() {
            Ok(conf) => conf,
            Err(e) => panic!("Could not transform active model into model: {e}"),
        };

        Self {
            db,
            server_config,
            prosody_config: None,
        }
    }

    fn prosody_config(&self) -> &ProsodyConfigFile {
        match &self.prosody_config {
            Some(config) => config,
            None => panic!("No config generated"),
        }
    }
}

// async fn setup_schema(db: &DbConn) -> Result<ExecResult, DbErr> {
//     // Setup Schema helper
//     let schema = Schema::new(DbBackend::Sqlite);

//     // Derive from Entity
//     let stmt: TableCreateStatement = schema.create_table_from_entity(server_config::Entity);

//     // Execute create table statement
//     db.execute(db.get_database_backend().build(&stmt)).await
// }
