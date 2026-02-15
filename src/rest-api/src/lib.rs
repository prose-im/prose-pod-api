// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod error;
pub mod extractors;
pub mod features;
pub mod forms;
pub mod responders;
pub mod util;

use std::sync::Arc;

use axum::{extract::FromRef, middleware, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use features::{factory_reset::restart_guard, startup_actions};
use service::{
    auth::AuthService,
    factory_reset::FactoryResetService,
    identity_provider::IdentityProvider,
    invitations::{invitation_service::InvitationApplicationService, InvitationRepository},
    licensing::LicensingService,
    members::{MemberService, UserApplicationService, UserRepository},
    models::DatabaseRwConnectionPools,
    network_checks::NetworkChecker,
    notifications::{notifier::email::EmailNotification, Notifier},
    pod_version::PodVersionService,
    prose_pod_server_service::ProsePodServerService,
    util::either::Either,
    workspace::WorkspaceService,
    xmpp::XmppService,
    AppConfig,
};
use tower::ServiceBuilder;
use tracing::{error, instrument};
use util::{error_catcher, LifecycleManager};

use crate::error::InvalidServerConfiguration;

pub trait AxumState: Clone + Send + Sync + 'static {}

// NOTE: Any Axum state must implement `Clone`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub base: MinimalAppState,
    pub db: DatabaseRwConnectionPools,
    pub app_config: Arc<AppConfig>,
    pub http_client: reqwest::Client,

    pub user_repository: UserRepository,
    pub invitation_repository: InvitationRepository,

    pub xmpp_service: XmppService,
    pub auth_service: AuthService,
    pub email_notifier: Option<Notifier<EmailNotification>>,
    pub member_service: MemberService,
    pub network_checker: NetworkChecker,
    pub workspace_service: WorkspaceService,
    pub licensing_service: LicensingService,
    pub pod_version_service: PodVersionService,
    pub factory_reset_service: FactoryResetService,
    pub prose_pod_server_service: ProsePodServerService,
    pub identity_provider: IdentityProvider,
    pub user_application_service: UserApplicationService,
    pub invitation_application_service: InvitationApplicationService,
}

impl AxumState for AppState {}

/// App state available even if the static configuration file is empty (useful)
/// on factory resets.
#[derive(Debug, Clone)]
pub struct MinimalAppState {
    pub lifecycle_manager: LifecycleManager,
    pub static_pod_version_service: PodVersionService,
}

impl AxumState for MinimalAppState {}

impl FromRef<AppState> for MinimalAppState {
    fn from_ref(input: &AppState) -> Self {
        input.base.clone()
    }
}

#[derive(Debug)]
pub struct PreStartupRouter {
    next: Router,
    // NOTE: It doesn’t make sense to duplicate state, but this will go
    //   whenever we fix https://github.com/prose-im/prose-pod-api/issues/357
    //   or https://github.com/prose-im/prose-pod-api/issues/341.
    //   It’s just a temporary shortcut.
    minimal_app_state: MinimalAppState,
}

/// A custom [`Router`] with a default configuration.
///
/// This route returns a [`PreStartupRouter`], forcing one to invoke this route first
/// then [`run_startup_actions`] to get the [`Router`]. We do it in this order because
/// creating the router is a cheap operation and Axum validates that there are no
/// overlapping routes (failing fast if something’s wrong).
#[instrument(level = "trace", skip_all)]
pub fn make_router(app_state: &AppState) -> PreStartupRouter {
    let minimal_app_state = app_state.base.clone();

    let router = Router::new()
        .merge(features::router(app_state.clone()))
        // Include trace context as header into the response.
        .layer(OtelInResponseLayer::default())
        // Start OpenTelemetry trace on incoming request.
        .layer(OtelAxumLayer::default())
        // See <https://github.com/prose-im/prose-pod-api/blob/c95e95677160ca5c27452bb0d68641a3bf2edff7/crates/rest-api/src/lib.rs#L70-L73>.
        .layer(ServiceBuilder::new().map_response(error_catcher))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            restart_guard,
        ));

    PreStartupRouter {
        next: router,
        minimal_app_state,
    }
}

/// A router used only after a factory reset, when the static configuration file
/// is empty and the API cannot start. Call `POST /reload` after editing this
/// file to (re)start the API.
#[instrument(level = "trace", skip_all)]
pub fn factory_reset_router(app_state: &MinimalAppState) -> Router {
    #[allow(unused_mut)]
    let mut router = Router::new()
        .merge(features::reload::factory_reset_router(app_state.clone()))
        .merge(features::version::minimal_router(app_state.clone()))
        .merge(features::health_check::minimal_router(app_state.clone()))
        // Include trace context as header into the response.
        .layer(OtelInResponseLayer::default())
        // Start OpenTelemetry trace on incoming request.
        .layer(OtelAxumLayer::default())
        // See <https://github.com/prose-im/prose-pod-api/blob/c95e95677160ca5c27452bb0d68641a3bf2edff7/crates/rest-api/src/lib.rs#L70-L73>.
        .layer(ServiceBuilder::new().map_response(error_catcher))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            restart_guard,
        ));

    #[cfg(all(debug_assertions, feature = "openapi"))]
    {
        router = router.merge(features::api_docs::router());
    }

    router
}

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error("Startup error: {0}")]
pub struct StartupError(String);

/// Run startup actions to ensure everything works correctly when the API launches.
///
/// This function acts as a state machine transition.
pub async fn run_startup_actions(
    router: PreStartupRouter,
    app_state: AppState,
) -> Result<Router, StartupError> {
    match startup_actions::run_startup_actions(app_state).await {
        Ok(Either::E1(())) => Ok(router.next),
        Ok(Either::E2(InvalidServerConfiguration(_))) => {
            // NOTE: The factory reset router does what we want, let’s take
            //   a shortcut and not create another one. This will go when we
            //   implement https://github.com/prose-im/prose-pod-api/issues/357
            //   anyway.
            Ok(factory_reset_router(&router.minimal_app_state))
        }
        Err(err) => Err(StartupError(err)),
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for AppState {
    type Target = MinimalAppState;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
