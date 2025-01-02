// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate rocket;

pub mod error;
pub mod features;
pub mod forms;
pub mod guards;
pub mod models;
pub mod responders;

use axum::{http::StatusCode, routing::get_service, Router};
use rocket::{
    catch, catchers, fairing::AdHoc, fs::FileServer, http::Status, Build, Request, Rocket,
};
use service::{
    auth::AuthService,
    dependencies::Uuid,
    network_checks::NetworkChecker,
    notifications::dependencies::Notifier,
    sea_orm::DatabaseConnection,
    secrets::SecretsStore,
    xmpp::{ServerCtl, XmppServiceInner},
    AppConfig,
};
use tower_http::services::ServeDir;
use tracing::error;

use self::error::Error;
use self::features::startup_actions::sequential_fairings;

pub trait AxumState: Clone + Send + Sync + 'static {}

/// A custom `Rocket` with a default configuration.
pub fn custom_rocket(
    rocket: Rocket<Build>,
    config: AppConfig,
    server_ctl: ServerCtl,
    xmpp_service: XmppServiceInner,
    auth_service: AuthService,
    notifier: Notifier,
    secrets_store: SecretsStore,
    network_checker: NetworkChecker,
) -> Rocket<Build> {
    rocket
        .attach(AdHoc::try_on_ignite(
            // NOTE: Fairings run in parallel, which means order is not guaranteed
            //   and race conditions could happen. This fairing runs all the fairings we need,
            //   one after another (since they all depend on the previous one).
            "Sequential fairings",
            |rocket| async {
                match sequential_fairings(&rocket).await {
                    Ok(()) => Ok(rocket),
                    Err(err) => {
                        error!("{err}");
                        Err(rocket)
                    }
                }
            },
        ))
        .mount("/", features::routes())
        .mount("/api-docs", FileServer::from("static/api-docs"))
        .register("/", catchers![default_catcher])
        .manage(Uuid::from_config(&config))
        .manage(config)
        .manage(server_ctl)
        .manage(xmpp_service)
        .manage(auth_service)
        .manage(notifier)
        .manage(secrets_store)
        .manage(network_checker)
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> Error {
    error::HTTPStatus(StatusCode::from_u16(status.code).unwrap()).into()
}

#[derive(Debug, Clone)]
pub struct AppState {
    db: DatabaseConnection,
    app_config: AppConfig,
    server_ctl: ServerCtl,
    xmpp_service: XmppServiceInner,
    auth_service: AuthService,
    notifier: Notifier,
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
        notifier: Notifier,
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
            notifier,
            secrets_store,
            network_checker,
        }
    }
}

impl AxumState for AppState {}

/// A custom [`Router`] with a default configuration.
pub fn custom_router(router: Router<AppState>, app_state: AppState) -> Router<AppState> {
    // on_startup(&app_state).await?;
    router
        .merge(features::router())
        .nest_service(
            "/api-docs",
            get_service(ServeDir::new("static/api-docs")).handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        // .register("/", catchers![default_catcher])
        .with_state(app_state)
}
