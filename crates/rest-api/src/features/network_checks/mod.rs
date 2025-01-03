// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod check_all;
mod check_dns_records;
mod check_ip_connectivity;
mod check_ports_reachability;
mod guards;
mod model;
mod util;

mod prelude {
    pub use std::convert::Infallible;

    pub use axum::{
        extract::{Query, State},
        response::{sse::Event, Sse},
        Json,
    };
    pub use futures::Stream;
    pub use serde::{Deserialize, Serialize};
    pub use serde_with::serde_as;
    pub use service::network_checks::*;

    pub use crate::{
        error::Error, forms, impl_network_check_event_from, impl_network_check_result_from,
        AppState,
    };
}

use axum::routing::get;
use axum_extra::handler::HandlerCallWithExtractors as _;

use crate::util::content_type_or::*;

pub use self::check_all::*;
pub use self::check_dns_records::*;
pub use self::check_ip_connectivity::*;
pub use self::check_ports_reachability::*;
pub use self::model::*;

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route(
            "/v1/network/checks",
            get(with_content_type::<TextEventStream, _>(
                check_network_configuration_stream_route,
            )
            .or(check_network_configuration_route)),
        )
        .route(
            "/v1/network/checks/dns",
            get(
                with_content_type::<TextEventStream, _>(check_dns_records_stream_route)
                    .or(check_dns_records_route),
            ),
        )
        .route(
            "/v1/network/checks/ip",
            get(
                with_content_type::<TextEventStream, _>(check_ip_stream_route)
                    .or(check_ip_route),
            ),
        )
        .route(
            "/v1/network/checks/ports",
            get(
                with_content_type::<TextEventStream, _>(check_ports_stream_route)
                    .or(check_ports_route),
            ),
        )
}
