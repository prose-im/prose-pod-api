// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Either<A, B> {
    pub fn left(&self) -> Option<&A> {
        match self {
            Self::Left(v) => Some(v),
            _ => None,
        }
    }
    pub fn right(&self) -> Option<&B> {
        match self {
            Self::Right(v) => Some(v),
            _ => None,
        }
    }
}

impl<A: Display, B: Display> Display for Either<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Either::Left(v) => Display::fmt(&v, f),
            Either::Right(v) => Display::fmt(&v, f),
        }
    }
}
