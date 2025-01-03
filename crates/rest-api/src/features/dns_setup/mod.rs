// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_dns_records;

use axum::routing::get;

pub use self::get_dns_records::*;

pub(super) fn router() -> axum::Router<crate::AppState> {
    axum::Router::new().route("/v1/network/dns/records", get(get_dns_records_route))
}
