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
    pub use service::{features::network_checks::*, model::XmppConnectionType};

    pub use crate::{
        error::Error, forms, guards::LazyGuard, impl_network_check_event_from,
        impl_network_check_result_from,
    };
}

pub use check_all::*;
pub use check_dns_records::*;
pub use check_ip_connectivity::*;
pub use check_ports_reachability::*;
pub use model::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        check_network_configuration_route,
        check_dns_records_route,
        check_dns_records_stream_route,
        check_ip_route,
        check_ip_stream_route,
        check_ports_route,
        check_ports_stream_route,
    ]
}
