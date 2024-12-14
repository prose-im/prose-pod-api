// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod check_all;
mod check_dns_records;
mod check_ip_connectivity;
mod check_ports_reachability;
mod model;
mod util;

mod prelude {
    pub use rocket::{
        response::stream::{Event, EventStream},
        serde::json::Json,
        State,
    };
    pub use serde::{Deserialize, Serialize};
    pub use serde_with::serde_as;
    pub use service::{network_checks::*, AppConfig};

    pub use crate::{
        error::Error, forms, guards::LazyGuard, impl_network_check_event_from,
        impl_network_check_result_from,
    };
}

use axum::routing::get;

pub use self::check_all::*;
pub use self::check_dns_records::*;
pub use self::check_ip_connectivity::*;
pub use self::check_ports_reachability::*;
pub use self::model::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        check_network_configuration_route,
        check_network_configuration_stream_route,
        check_dns_records_route,
        check_dns_records_stream_route,
        check_ip_route,
        check_ip_stream_route,
        check_ports_route,
        check_ports_stream_route,
    ]
}

pub(super) fn router<S: crate::AxumState>() -> axum::Router<S> {
    axum::Router::new()
        .route(
            "/v1/network/checks",
            get(check_network_configuration_route_axum),
        )
        .route(
            "/v1/network/checks",
            get(check_network_configuration_stream_route_axum),
        )
        .route("/v1/network/checks/dns", get(check_dns_records_route_axum))
        .route(
            "/v1/network/checks/dns",
            get(check_dns_records_stream_route_axum),
        )
        .route("/v1/network/checks/ip", get(check_ip_route_axum))
        .route("/v1/network/checks/ip", get(check_ip_stream_route_axum))
        .route("/v1/network/checks/ports", get(check_ports_route_axum))
        .route(
            "/v1/network/checks/ports",
            get(check_ports_stream_route_axum),
        )
}
