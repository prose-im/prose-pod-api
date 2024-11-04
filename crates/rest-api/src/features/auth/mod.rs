// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod login;

pub use login::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![login_route]
}
