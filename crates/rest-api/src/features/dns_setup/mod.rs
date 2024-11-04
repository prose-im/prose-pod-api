// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod get_dns_records;

pub use get_dns_records::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![get_dns_records_route]
}
