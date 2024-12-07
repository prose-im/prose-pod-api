// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

pub mod error;
pub mod features;
pub mod forms;
pub mod guards;
pub mod models;
pub mod responders;

use error::Error;
use features::startup_actions::sequential_fairings;
use rocket::{fairing::AdHoc, fs::FileServer, http::Status, Build, Request, Rocket};
use service::{
    auth::AuthService,
    dependencies::Uuid,
    network_checks::NetworkChecker,
    notifications::dependencies::Notifier,
    secrets::SecretsStore,
    xmpp::{ServerCtl, XmppServiceInner},
    AppConfig,
};
use tracing::error;

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
    error::HTTPStatus(status).into()
}
