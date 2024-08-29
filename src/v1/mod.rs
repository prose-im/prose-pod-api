// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod init;
pub mod invitations;
pub mod members;
pub mod network;
pub mod routes;
pub mod server;
pub mod workspace;

pub use routes::*;

use rocket::{response::status, serde::json::Json, Route};

use crate::error::Error;

pub(super) fn routes() -> Vec<Route> {
    vec![
        routes![login],
        invitations::routes(),
        members::routes(),
        server::routes(),
        workspace::routes(),
        init::routes(),
        network::routes(),
    ]
    .concat()
}

pub type R<T> = Result<Json<T>, Error>;
pub type Created<T> = Result<status::Created<Json<T>>, Error>;
