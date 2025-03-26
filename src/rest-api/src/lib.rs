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

use axum::{http::StatusCode, routing::get_service, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use features::startup_actions;
use service::{
    auth::AuthService,
    dependencies::Uuid,
    network_checks::NetworkChecker,
    notifications::{notifier::email::EmailNotification, Notifier},
    sea_orm::DatabaseConnection,
    secrets::SecretsStore,
    xmpp::{ServerCtl, XmppServiceInner},
    AppConfig,
};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tracing::{error, instrument};
use util::error_catcher;

pub trait AxumState: Clone + Send + Sync + 'static {}

#[derive(Debug, Clone)]
pub struct AppState {
    db: DatabaseConnection,
    app_config: AppConfig,
    server_ctl: ServerCtl,
    xmpp_service: XmppServiceInner,
    auth_service: AuthService,
    email_notifier: Option<Notifier<EmailNotification>>,
    secrets_store: SecretsStore,
    network_checker: NetworkChecker,
    uuid_gen: Uuid,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        app_config: AppConfig,
        server_ctl: ServerCtl,
        xmpp_service: XmppServiceInner,
        auth_service: AuthService,
        email_notifier: Option<Notifier<EmailNotification>>,
        secrets_store: SecretsStore,
        network_checker: NetworkChecker,
    ) -> Self {
        Self {
            db,
            uuid_gen: Uuid::from_config(&app_config),
            app_config,
            server_ctl,
            xmpp_service,
            auth_service,
            email_notifier,
            secrets_store,
            network_checker,
        }
    }
}

impl AxumState for AppState {}

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
        .nest_service(
            "/api-docs",
            get_service(ServeDir::new("static/api-docs")).handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        // Include trace context as header into the response.
        .layer(OtelInResponseLayer::default())
        // Start OpenTelemetry trace on incoming request.
        .layer(OtelAxumLayer::default())
        // See <https://github.com/prose-im/prose-pod-api/blob/c95e95677160ca5c27452bb0d68641a3bf2edff7/crates/rest-api/src/lib.rs#L70-L73>.
        .layer(ServiceBuilder::new().map_response(error_catcher));

    PreStartupRouter(router)
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
    startup_actions::run_startup_actions(&app_state)
        .await
        .map_err(StartupError)?;

    Ok(router.0)
}
