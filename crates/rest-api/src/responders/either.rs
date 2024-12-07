// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use rocket::{
    response::{self, Responder},
    Request,
};

pub struct Either<A, B>(service::util::Either<A, B>);

impl<A, B> Either<A, B> {
    pub fn left(value: A) -> Self {
        Self(service::util::Either::Left(value))
    }
    pub fn right(value: B) -> Self {
        Self(service::util::Either::Right(value))
    }
}

impl<A, B> Deref for Either<A, B> {
    type Target = service::util::Either<A, B>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r, 'o: 'r, A: Responder<'r, 'o>, B: Responder<'r, 'o>> Responder<'r, 'o> for Either<A, B> {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
        match self.0 {
            service::util::Either::Left(value) => value.respond_to(request),
            service::util::Either::Right(value) => value.respond_to(request),
        }
    }
}
