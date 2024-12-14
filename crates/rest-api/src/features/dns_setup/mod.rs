// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_dns_records;

use axum::routing::get;

pub use self::get_dns_records::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![get_dns_records_route]
}

pub(super) fn router<S: crate::AxumState>() -> axum::Router<S> {
    axum::Router::new().route("/v1/network/dns/records", get(get_dns_records_route_axum))
}
