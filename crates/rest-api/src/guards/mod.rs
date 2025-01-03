// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod db;
mod notifier;
mod pod_network_config;
mod server_config;
mod server_manager;
mod unauthenticated_server_manager;
mod util;
mod uuid_generator;
mod xmpp_service;

use std::ops::Deref;

pub use db::*;
pub use unauthenticated_server_manager::*;

use prelude::*;
use rocket::http::Status;

pub mod prelude {
    pub use super::util::*;
    pub use super::LazyFromRequest;
    pub use crate::error::{self, Error};
    pub use crate::request_state;
    pub use rocket::{outcome::try_outcome, request::Outcome, Request};
}

use crate::error::{self, Error};

#[repr(transparent)]
pub struct LazyGuard<Inner> {
    // NOTE: We have to wrap model in a `Result` instead of sending `Outcome::Error`
    //   because when sending `Outcome::Error((Status::BadRequest, Error::PodNotInitialized))`
    //   [Rocket's built-in catcher] doesn't use `impl Responder for Error` but instead
    //   transforms the response to HTML (no matter the `Accept` header, which is weird)
    //   saying "The request could not be understood by the server due to malformed syntax.".
    //   We can't build our own [error catcher] as it does not have access to the error
    //   sent via `Outcome::Error`.
    //
    //   [Rocket's built-in catcher]: https://rocket.rs/v0.5/guide/requests/#built-in-catcher
    //   [error catcher]: https://rocket.rs/v0.5/guide/requests/#error-catchers
    pub inner: Result<Inner, Error>,
}

impl<Inner> Deref for LazyGuard<Inner> {
    type Target = Result<Inner, Error>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[rocket::async_trait]
pub trait LazyFromRequest<'r>: Sized {
    type Error;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error>;
}

#[rocket::async_trait]
impl<'r, Inner> rocket::request::FromRequest<'r> for LazyGuard<Inner>
where
    Inner: LazyFromRequest<'r>,
    <Inner as LazyFromRequest<'r>>::Error: Into<error::Error>,
{
    type Error = error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match Inner::from_request(req).await {
            Outcome::Success(s) => Outcome::Success(Self { inner: Ok(s) }),
            Outcome::Forward(f) => Outcome::Forward(f),
            Outcome::Error((_, e)) => Outcome::Success(Self {
                inner: Err(e.into()),
            }),
        }
    }
}

impl Into<(Status, Error)> for Error {
    fn into(self) -> (Status, Error) {
        (self.http_status, self)
    }
}

impl<S> Into<Outcome<S, Error>> for Error {
    fn into(self) -> Outcome<S, Error> {
        Outcome::Error(self.into())
    }
}
