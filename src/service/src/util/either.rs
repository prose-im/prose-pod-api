// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display};

macro_rules! gen {
    ($t:ident<$case_0:ident$(, $case:ident)*; $case_n:ident>) => {
        pub enum $t<$case_0$(, $case)*, $case_n> {
            $case_0($case_0),
            $($case($case),)*
            $case_n($case_n),
        }

        impl<$case_0: Debug$(, $case: Debug)*, $case_n: Debug> Debug for $t<$case_0$(, $case)*, $case_n> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::$case_0(v) => Debug::fmt(&v, f),
                    $(Self::$case(v) => Debug::fmt(&v, f),)*
                    Self::$case_n(v) => Debug::fmt(&v, f),
                }
            }
        }

        impl<$case_0: Display$(, $case: Display)*, $case_n: Display> Display for $t<$case_0$(, $case)*, $case_n> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::$case_0(v) => Display::fmt(&v, f),
                    $(Self::$case(v) => Display::fmt(&v, f),)*
                    Self::$case_n(v) => Display::fmt(&v, f),
                }
            }
        }

        // MARK: Useful (opinionated) implementations

        // NOTE: `EitherN<…, anyhow::Error>` -> `std::error::Error`
        //   (therefore `EitherN<…, anyhow::Error>` -> `anyhow::Error`).
        impl<$case_0$(, $case)*> std::error::Error for $t<$case_0$(, $case)*, anyhow::Error>
        where
            $case_0: std::error::Error + Send + Sync + 'static,
            $($case: std::error::Error + Send + Sync + 'static,)*
        {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    Self::$case_0(err) => err.source(),
                    $(Self::$case(err) => err.source(),)*
                    Self::$case_n(err) => err.source(),
                }
            }
        }
        // NOTE: `anyhow::Error` -> `EitherN<…, anyhow::Error>`.
        impl<$case_0$(, $case)*> From<anyhow::Error> for $t<$case_0$(, $case)*, anyhow::Error> {
            fn from(value: anyhow::Error) -> Self {
                Self::$case_n(value)
            }
        }
        // NOTE: `.context` for `EitherN<…, anyhow::Error>`.
        impl<$case_0$(, $case)*> Context for $t<$case_0$(, $case)*, anyhow::Error> {
            fn err_context<C>(self, context: C) -> Self
            where
                C: Display + Send + Sync + 'static,
            {
                match self {
                    Self::$case_n(err) => Self::$case_n(err.context(context)),
                    err => err,
                }
            }
        }
        // NOTE: `sea_orm::DbErr` -> `EitherN<…, anyhow::Error>`.
        impl<$case_0$(, $case)*> From<sea_orm::DbErr> for $t<$case_0$(, $case)*, anyhow::Error> {
            fn from(value: sea_orm::DbErr) -> Self {
                Self::$case_n(anyhow::Error::new(value).context("Database error"))
            }
        }
    };
}

// NOTE: `;` allows differentiating the last case for
//   `EitherN<…, anyhow::Error>` helpers.
gen!(Either<E1; E2>);
gen!(Either3<E1, E2; E3>);
gen!(Either4<E1, E2, E3; E4>);
// NOTE: We should not go higher. If you need `Either5`, use an `enum`.

pub trait Context {
    fn err_context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static;
}

// TODO: Implement `Context` if `anyhow::Error: From<Result<…>>`
//   (`map_err(anyhow::Error::new)`).
impl<T, E: Context> Context for Result<T, E> {
    fn err_context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(err.err_context(context)),
        }
    }
}

// MARK: - Helpers

pub fn to_either3_1_3<E1, E2, E3>(either: Either<E1, E3>) -> Either3<E1, E2, E3> {
    match either {
        Either::E1(val) => Either3::E1(val),
        Either::E2(val) => Either3::E3(val),
    }
}

pub fn to_either4_1_4<E1, E2, E3, E4>(either: Either<E1, E4>) -> Either4<E1, E2, E3, E4> {
    match either {
        Either::E1(val) => Either4::E1(val),
        Either::E2(val) => Either4::E4(val),
    }
}

// Either3 from 2..2

impl<E1, E2, E3> From<Either<E2, E3>> for Either3<E1, E2, E3> {
    fn from(value: Either<E2, E3>) -> Self {
        match value {
            Either::E1(err) => Self::E2(err),
            Either::E2(err) => Self::E3(err),
        }
    }
}

// Either4 from 2..3

impl<E1, E2, E3, E4> From<Either<E3, E4>> for Either4<E1, E2, E3, E4> {
    fn from(value: Either<E3, E4>) -> Self {
        match value {
            Either::E1(err) => Self::E3(err),
            Either::E2(err) => Self::E4(err),
        }
    }
}

impl<E1, E2, E3, E4> From<Either3<E2, E3, E4>> for Either4<E1, E2, E3, E4> {
    fn from(value: Either3<E2, E3, E4>) -> Self {
        match value {
            Either3::E1(err) => Self::E2(err),
            Either3::E2(err) => Self::E3(err),
            Either3::E3(err) => Self::E4(err),
        }
    }
}
