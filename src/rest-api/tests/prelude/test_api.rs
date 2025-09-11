// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum_test::TestServer;
use prose_pod_api::{util::LifecycleManager, AppState, MinimalAppState};
use service::xmpp::ServerCtlImpl as _;
use tracing::*;

use super::{steps::app_config::reload_config, test_world::TestWorld};

pub async fn test_server(world: &mut TestWorld) -> anyhow::Result<TestServer> {
    info!("Creating test router…");

    if world.app_config.is_none() {
        reload_config(world);
    }

    let app_config = world.app_config();

    // Create API XMPP account
    // NOTE: This is done automatically via Prosody, we need to do it by hand here.
    if let Err(err) = world
        .mock_server_ctl
        .add_user(
            &app_config.api_jid(),
            &app_config.bootstrap.prose_pod_api_xmpp_password,
        )
        .await
    {
        panic!("Could not create API XMPP account: {}", err);
    }

    let lifecycle_manager = LifecycleManager::new();
    let app_state = AppState::new(
        MinimalAppState {
            lifecycle_manager: lifecycle_manager.clone(),
            secrets_store: world.secrets_store().clone(),
            static_pod_version_service: world.pod_version_service.clone(),
        },
        world.db.clone(),
        app_config,
        world.server_ctl.clone(),
        world.xmpp_service.clone(),
        world.auth_service.clone(),
        Some(world.email_notifier.clone()),
        world.network_checker.clone(),
        world.license_service().clone(),
        world.pod_version_service.clone(),
    );

    let router = prose_pod_api::make_router(&app_state);
    let app = prose_pod_api::run_startup_actions(router, app_state).await?;
    (lifecycle_manager.wait_for_startup_actions_to_finish()).await?;
    Ok(TestServer::new(app).expect("Could not create test server."))
}
