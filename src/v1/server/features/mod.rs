// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod compliance_tests;
pub mod config;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    vec![
        compliance_tests::routes(),
        config::routes(),
    ]
    .concat()
}
