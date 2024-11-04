// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod redoc;

pub use redoc::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![redoc_route]
}
