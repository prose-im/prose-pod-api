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
use features::startup_actions::on_startup;
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
use tracing::error;
use util::error_catcher;

pub trait AxumState: Clone + Send + Sync + 'static {}

#[derive(Debug, Clone)]
pub struct AppState {
    db: DatabaseConnection,
    app_config: AppConfig,
    server_ctl: ServerCtl,
    xmpp_service: XmppServiceInner,
    auth_service: AuthService,
    email_notifier: Notifier<EmailNotification>,
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
        email_notifier: Notifier<EmailNotification>,
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

#[derive(Debug, thiserror::Error)]
#[error("Startup error: {0}")]
pub struct StartupError(String);

/// A custom [`Router`] with a default configuration.
pub async fn custom_router(app_state: AppState) -> Result<Router, StartupError> {
    // Create the router first, because it's a cheap operation and
    // Axum validates that there are no overlapping routes.
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
        // See <https://github.com/prose-im/prose-pod-api/blob/c95e95677160ca5c27452bb0d68641a3bf2edff7/crates/rest-api/src/lib.rs#L70-L73>.
        .layer(ServiceBuilder::new().map_response(error_catcher));

    // Before returning the router, run startup actions to ensure
    // everything works correctly when the API launches.
    //
    // If we did that in `main`, we'd have to add it in both the tests' `main`
    // function and in the API's `main` function.
    //
    // Instead of relying on integration tests to detect that we forgot to
    // run startup actions in the API's `main` function, we can do it
    // in this shared code.
    on_startup(&app_state).await.map_err(StartupError)?;

    Ok(router)
}
