// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

use prose_pod_api::custom_rocket;
use prose_pod_api::guards::JWTService;
use service::notifier::Notifier;
use service::ServerCtl;
use service::{notifier::EmailNotifier, prosody::ProsodyCtl};
use std::sync::{Arc, Mutex};

#[launch]
fn rocket() -> _ {
    let jwt_service = match JWTService::from_env() {
        Ok(service) => service,
        Err(err) => panic!("{err}"),
    };

    custom_rocket(rocket::build())
        .manage(jwt_service)
        .manage(ServerCtl::new(Arc::new(Mutex::new(ProsodyCtl::default()))))
        .manage(Notifier::new(Arc::new(Mutex::new(EmailNotifier))))
}
