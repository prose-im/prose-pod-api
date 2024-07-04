// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prosody;

use ::entity::{
    server_config::{self, Model as ServerConfig},
    workspace::{self},
};
use ::migration::{self, MigratorTrait};
use cucumber::World;
use sea_orm::*;
use service::{
    config::Config,
    prosody::ProsodyConfig,
    repositories::{ServerConfigRepository, WorkspaceRepository},
};

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
    app_config: Config,
    server_config: ServerConfig,
    prosody_config: Option<ProsodyConfig>,
}

impl TestWorld {
    async fn new() -> Self {
        let app_config = Config::figment();

        // Connecting SQLite
        let db = match Database::connect("sqlite::memory:").await {
            Ok(conn) => conn,
            Err(e) => panic!("Could not connect to test database: {e}"),
        };

        // Setup database schema
        if let Err(e) = migration::Migrator::up(&db, None).await {
            panic!("Could not setup test database schema: {e}");
        }

        let server_config = server_config::ActiveModel {
            domain: Set("prose.test.org".to_string()),
            ..Default::default()
        };
        let server_config = match ServerConfigRepository::create(&db, server_config).await {
            Ok(conf) => conf,
            Err(e) => panic!("Could not create server config: {e}"),
        };

        let workspace = workspace::ActiveModel {
            name: Set(DEFAULT_WORKSPACE_NAME.to_string()),
            ..Default::default()
        };
        let _workspace = match WorkspaceRepository::create(&db, workspace).await {
            Ok(conf) => conf,
            Err(e) => panic!("Could not create workspace: {e}"),
        };

        Self {
            db,
            app_config,
            server_config,
            prosody_config: None,
        }
    }

    fn prosody_config(&self) -> &ProsodyConfig {
        match &self.prosody_config {
            Some(config) => config,
            None => panic!("No config generated"),
        }
    }
}
