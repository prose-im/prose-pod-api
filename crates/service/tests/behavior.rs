// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod prosody;

use std::{path::Path, str::FromStr as _};

use cucumber::World;
use sea_orm::*;
use service::{
    app_config::{AppConfig, CONFIG_FILE_NAME},
    init::WorkspaceCreateForm,
    models::JidDomain,
    prosody::ProsodyConfig,
    server_config::{entities::server_config, ServerConfigCreateForm, ServerConfigRepository},
    workspace::WorkspaceRepository,
    MigratorTrait as _,
};

pub const DEFAULT_WORKSPACE_NAME: &'static str = "Prose";
pub const DEFAULT_DOMAIN: &'static str = "prose.test.org";

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
    server_config: server_config::Model,
    prosody_config: Option<ProsodyConfig>,
}

impl TestWorld {
    async fn new() -> Self {
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let app_config = AppConfig::from_path(crate_root.join("tests").join(CONFIG_FILE_NAME));

        // Connecting SQLite
        let db = match Database::connect(&app_config.databases.main.url).await {
            Ok(conn) => conn,
            Err(e) => panic!("Could not connect to test database: {e}"),
        };

        // Setup database schema
        if let Err(e) = service::Migrator::up(&db, None).await {
            panic!("Could not setup test database schema: {e}");
        }

        let server_config = ServerConfigCreateForm {
            domain: JidDomain::from_str(DEFAULT_DOMAIN).unwrap(),
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
