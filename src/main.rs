// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate rocket;

use prose_pod_api::custom_rocket;
use prose_pod_api::guards::JWTService;
use service::prosody::ProsodyCtl;
use service::ServerCtl;
use std::sync::{Arc, Mutex};

#[launch]
fn rocket() -> _ {
    custom_rocket(rocket::build())
        .manage(JWTService::from_env())
        .manage(ServerCtl::new(Arc::new(Mutex::new(ProsodyCtl::default()))))
}
