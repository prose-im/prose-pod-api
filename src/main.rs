// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

use prose_pod_api::custom_rocket;
use prose_pod_api::guards::JWTService;
use service::config::Config;
use service::dependencies::Notifier;
use service::prosody::ProsodyAdminRest;
use service::ServerCtl;
use std::sync::{Arc, Mutex};

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let config = Config::figment();
    let jwt_service = match JWTService::from_env() {
        Ok(service) => service,
        Err(err) => panic!("{err}"),
    };
    let notifier = Notifier::from_config(&config).unwrap_or_else(|e| panic!("{e}"));

    custom_rocket(rocket::build(), &config)
        .manage(jwt_service)
        .manage(ServerCtl::new(Arc::new(Mutex::new(
            ProsodyAdminRest::from_config(&config),
        ))))
        .manage(notifier)
}
