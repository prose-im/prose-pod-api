// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prosody;

use cucumber::World;
use migration::{self, MigratorTrait};
use sea_orm::*;
use service::{
    config::Config,
    controllers::init_controller::WorkspaceCreateForm,
    entity::server_config,
    prosody::ProsodyConfig,
    repositories::{ServerConfigCreateForm, ServerConfigRepository, WorkspaceRepository},
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
    server_config: server_config::Model,
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

        let server_config = ServerConfigCreateForm {
            domain: "prose.test.org".to_string(),
        };
        let server_config = match ServerConfigRepository::create(&db, server_config).await {
            Ok(conf) => conf,
            Err(e) => panic!("Could not create server config: {e}"),
        };

        let workspace = WorkspaceCreateForm {
            name: DEFAULT_WORKSPACE_NAME.to_string(),
            accent_color: None,
        };
        if let Err(err) = WorkspaceRepository::create(&db, workspace).await {
            panic!("Could not create workspace: {err}")
        }

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
