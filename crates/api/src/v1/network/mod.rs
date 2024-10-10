// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod checks;
pub mod dns;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    vec![
        dns::routes(),
        checks::routes(),
    ]
    .concat()
}
