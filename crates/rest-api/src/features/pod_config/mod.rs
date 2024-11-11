// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod pod_address;
mod pod_config;

pub use pod_address::*;
pub use pod_config::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        get_pod_config_route,
        set_pod_address_route,
        get_pod_address_route,
    ]
}
