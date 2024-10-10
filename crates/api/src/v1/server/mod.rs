// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod backup;
pub mod compliance_tests;
pub mod config;
pub mod connection_history;
pub mod data;
pub mod insights;
pub mod logs;
pub mod network;
pub mod security;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    vec![
        backup::routes(),
        compliance_tests::routes(),
        config::routes(),
        connection_history::routes(),
        data::routes(),
        insights::routes(),
        logs::routes(),
        network::routes(),
        security::routes(),
    ]
    .concat()
}
