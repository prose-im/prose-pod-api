// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum_test::TestServer;
use prose_pod_api::{util::LifecycleManager, AppState};
use tracing::*;

use super::test_world::TestWorld;

pub async fn test_server(world: &TestWorld) -> anyhow::Result<TestServer> {
    info!("Creating test router…");

    let lifecycle_manager = LifecycleManager::new();
    let app_state = AppState::new(
        world.db.clone(),
        world.app_config.read().unwrap().to_owned(),
        world.server_ctl.clone(),
        world.xmpp_service.clone(),
        world.auth_service.clone(),
        Some(world.email_notifier.clone()),
        world.secrets_store.clone(),
        world.network_checker.clone(),
        lifecycle_manager.clone(),
    );

    let router = prose_pod_api::make_router(&app_state);
    let app = prose_pod_api::run_startup_actions(router, app_state).await?;
    (lifecycle_manager.wait_for_startup_actions_to_finish()).await?;
    Ok(TestServer::new(app).expect("Could not create test server."))
}
