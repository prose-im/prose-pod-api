// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod invites;
pub mod members;
pub mod routes;
pub mod server;
pub mod workspace;

pub use routes::*;

use rocket::Route;

pub(super) fn routes() -> Vec<Route> {
    vec![
        routes![openapi, init, login],
        invites::routes(),
        members::routes(),
        server::routes(),
        workspace::routes(),
    ]
    .concat()
}
