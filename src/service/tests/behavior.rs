// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prosody;

use std::path::Path;

use cucumber::World;
use sea_orm::*;
use service::{
    app_config::{AppConfig, CONFIG_FILE_NAME},
    prosody::ProsodyConfig,
    MigratorTrait as _,
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
    app_config: AppConfig,
    prosody_config: Option<ProsodyConfig>,
}

impl TestWorld {
    async fn new() -> Self {
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config_path = crate_root.join("tests").join(CONFIG_FILE_NAME);
        let app_config = AppConfig::from_path(&config_path)
            .expect(&format!("Invalid config file at {}", config_path.display()));

        // Connecting SQLite
        let db = match Database::connect(&app_config.api.databases.main.url).await {
            Ok(conn) => conn,
            Err(e) => panic!("Could not connect to test database: {e}"),
        };

        // Setup database schema
        if let Err(e) = service::Migrator::up(&db, None).await {
            panic!("Could not setup test database schema: {e}");
        }

        Self {
            db,
            app_config,
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
