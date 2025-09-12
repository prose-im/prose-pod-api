// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod check_all;
mod check_dns_records;
mod check_ip_connectivity;
mod check_ports_reachability;
mod extractors;
mod model;
mod util;

mod prelude {
    pub use std::{convert::Infallible, sync::Arc};

    pub use axum::{
        extract::{Query, State},
        http::{header::ACCEPT, HeaderMap},
        response::{sse::Event, Sse},
        Json,
    };
    pub use axum_extra::either::Either;
    pub use futures::Stream;
    pub use mime::TEXT_EVENT_STREAM;
    pub use serde_with::serde_as;
    pub use serdev::Serialize;
    pub use service::network_checks::*;

    pub(crate) use crate::AppConfig;
    pub use crate::{
        error::Error, forms, impl_network_check_event_from, impl_network_check_result_from,
        util::headers_ext::HeaderValueExt as _, AppState,
    };
}

use std::time::Duration;

use axum::middleware::from_extractor_with_state;
use axum::routing::get;
use lazy_static::lazy_static;
use service::auth::IsAdmin;

use crate::AppState;

pub use self::check_all::*;
pub use self::check_dns_records::*;
pub use self::check_ip_connectivity::*;
pub use self::check_ports_reachability::*;
pub use self::model::*;

use super::NETWORK_ROUTE;

lazy_static! {
    static ref SSE_TIMEOUT: Duration = Duration::from_secs(5 * 60);
}

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            NETWORK_ROUTE,
            axum::Router::new()
                .route("/checks", get(check_network_configuration_route))
                .route("/checks/dns", get(check_dns_records_route))
                .route("/checks/ip", get(check_ip_route))
                .route("/checks/ports", get(check_ports_route)),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}
