// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod error;
pub mod features;
pub mod forms;
pub mod guards;
pub mod responders;
pub mod util;

use std::sync::{Arc, RwLock};

use axum::{extract::FromRef, middleware, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use features::{factory_reset::restart_guard, startup_actions};
use service::{
    auth::AuthService,
    dependencies::Uuid,
    licensing::LicenseService,
    network_checks::NetworkChecker,
    notifications::{notifier::email::EmailNotification, Notifier},
    sea_orm::DatabaseConnection,
    secrets::SecretsStore,
    xmpp::{ServerCtl, XmppServiceInner},
    AppConfig,
};
use tower::ServiceBuilder;
use tracing::{error, instrument};
use util::{error_catcher, LifecycleManager};

pub trait AxumState: Clone + Send + Sync + 'static {}

#[derive(Debug, Clone)]
pub struct AppState {
    base: MinimalAppState,
    db: DatabaseConnection,
    app_config: Arc<RwLock<AppConfig>>,
    server_ctl: ServerCtl,
    xmpp_service: XmppServiceInner,
    auth_service: AuthService,
    email_notifier: Option<Notifier<EmailNotification>>,
    secrets_store: SecretsStore,
    network_checker: NetworkChecker,
    license_service: LicenseService,
    uuid_gen: Uuid,
}

impl AppState {
    pub fn new(
        base: MinimalAppState,
        db: DatabaseConnection,
        app_config: Arc<RwLock<AppConfig>>,
        server_ctl: ServerCtl,
        xmpp_service: XmppServiceInner,
        auth_service: AuthService,
        email_notifier: Option<Notifier<EmailNotification>>,
        secrets_store: SecretsStore,
        network_checker: NetworkChecker,
        license_service: LicenseService,
    ) -> Self {
        let uuid_gen = Uuid::from_config(&app_config.read().unwrap());
        Self {
            base,
            db,
            uuid_gen,
            app_config,
            server_ctl,
            xmpp_service,
            auth_service,
            email_notifier,
            secrets_store,
            network_checker,
            license_service,
        }
    }

    /// An immutable version of the global app configuration.
    pub fn app_config_frozen(&self) -> AppConfig {
        self.app_config.read().unwrap().clone()
    }
}

impl AxumState for AppState {}

/// App state available even if the static configuration file is empty (useful)
/// on factory resets.
#[derive(Debug, Clone)]
pub struct MinimalAppState {
    pub lifecycle_manager: LifecycleManager,
}

impl AxumState for MinimalAppState {}

impl FromRef<AppState> for MinimalAppState {
    fn from_ref(input: &AppState) -> Self {
        input.base.clone()
    }
}

pub struct PreStartupRouter(Router);

/// A custom [`Router`] with a default configuration.
///
/// This route returns a [`PreStartupRouter`], forcing one to invoke this route first
/// then [`run_startup_actions`] to get the [`Router`]. We do it in this order because
/// creating the router is a cheap operation and Axum validates that there are no
/// overlapping routes (failing fast if something’s wrong).
#[instrument(level = "trace", skip_all)]
pub fn make_router(app_state: &AppState) -> PreStartupRouter {
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

    PreStartupRouter(router)
}

/// A router used only sfter a factory reset, when the static configuration file
/// is empty and the API cannot start. Call `POST /reload` after editing this
/// file to (re)start the API.
#[instrument(level = "trace", skip_all)]
pub fn factory_reset_router(app_state: &MinimalAppState) -> Router {
    Router::new()
        .merge(features::api_docs::router())
        .merge(features::reload::factory_reset_router(app_state.clone()))
        .merge(features::version::router())
        // Include trace context as header into the response.
        .layer(OtelInResponseLayer::default())
        // Start OpenTelemetry trace on incoming request.
        .layer(OtelAxumLayer::default())
        // See <https://github.com/prose-im/prose-pod-api/blob/c95e95677160ca5c27452bb0d68641a3bf2edff7/crates/rest-api/src/lib.rs#L70-L73>.
        .layer(ServiceBuilder::new().map_response(error_catcher))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            restart_guard,
        ))
}

#[derive(Debug, thiserror::Error)]
#[error("Startup error: {0}")]
pub struct StartupError(String);

/// Run startup actions to ensure everything works correctly when the API launches.
///
/// This function acts as a state machine transition.
pub async fn run_startup_actions(
    router: PreStartupRouter,
    app_state: AppState,
) -> Result<Router, StartupError> {
    startup_actions::run_startup_actions(app_state)
        .await
        .map_err(StartupError)?;

    Ok(router.0)
}
