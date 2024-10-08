// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod routes;

pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    routes![
        check_network_configuration,
        check_dns_records_stream,
        check_ports_stream,
        check_ip_stream,
    ]
}
