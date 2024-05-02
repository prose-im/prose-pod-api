// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod db;
mod jid;
mod jwt_service;
mod server_config;
mod server_manager;

pub use db::*;
pub use jid::*;
pub use jwt_service::*;
pub use server_config::*;
pub use server_manager::*;

use rocket::http::Status;
use rocket::request::Outcome;

use crate::error::Error;

impl Into<(Status, Error)> for Error {
    fn into(self) -> (Status, Error) {
        (self.http_status(), self)
    }
}

impl<S> Into<Outcome<S, Error>> for Error {
    fn into(self) -> Outcome<S, Error> {
        Outcome::Error(self.into())
    }
}
