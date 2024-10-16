// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod config;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    vec![config::routes()].concat()
}
