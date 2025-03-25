// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
