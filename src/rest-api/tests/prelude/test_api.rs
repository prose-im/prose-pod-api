// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum_test::TestServer;
use prose_pod_api::{util::LifecycleManager, AppState, MinimalAppState};
use tracing::*;

use super::{app_config::reload_config, test_world::TestWorld};

pub async fn test_server(world: &mut TestWorld) -> anyhow::Result<TestServer> {
    info!("Creating test router…");

    if world.app_config.is_none() {
        reload_config(world);
    }

    let app_config = world.app_config();

    let lifecycle_manager = LifecycleManager::new();
    let app_state = AppState {
        base: MinimalAppState {
            lifecycle_manager: lifecycle_manager.clone(),
            secrets_store: world.secrets_store().clone(),
            static_pod_version_service: world.pod_version_service.clone(),
        },
        db: world.db.clone(),
        app_config,
        user_repository: world.user_repository().clone(),
        invitation_repository: world.invitation_repository().clone(),
        xmpp_service: world.xmpp_service().clone(),
        auth_service: world.auth_service().clone(),
        email_notifier: Some(world.email_notifier.clone()),
        network_checker: world.network_checker.clone(),
        workspace_service: world.workspace_service().clone(),
        licensing_service: world.licensing_service().clone(),
        pod_version_service: world.pod_version_service.clone(),
        factory_reset_service: world.factory_reset_service.clone(),
        prose_pod_server_service: world.server_service().clone(),
        identity_provider: world.identity_provider.clone(),
        user_application_service: world.user_application_service().clone(),
        invitation_application_service: world.invitation_application_service().clone(),
    };

    let router = prose_pod_api::make_router(&app_state);
    let app = prose_pod_api::run_startup_actions(router, app_state).await?;
    (lifecycle_manager.wait_for_startup_actions_to_finish()).await?;
    Ok(TestServer::new(app).expect("Could not create test server."))
}
