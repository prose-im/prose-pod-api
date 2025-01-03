// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum_test::TestServer;
use prose_pod_api::AppState;
use tracing::*;

use super::test_world::TestWorld;

pub async fn test_server(world: &TestWorld) -> TestServer {
    debug!("Creating test router…");

    let app_state = AppState::new(
        world.db.clone(),
        world.app_config.clone(),
        world.server_ctl.clone(),
        world.xmpp_service.clone(),
        world.auth_service.clone(),
        world.notifier.clone(),
        world.secrets_store.clone(),
        world.network_checker.clone(),
    );

    let router = prose_pod_api::custom_router(app_state);
    TestServer::new(router).expect("Could not create test server.")
}
