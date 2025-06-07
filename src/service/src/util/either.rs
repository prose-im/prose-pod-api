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

        // MARK: USEFUL (OPINIONATED) IMPLEMENTATIONS

        // NOTE: `anyhow::Error` -> `EitherN<…, anyhow::Error>`.
        impl<$case_0$(, $case)*> From<anyhow::Error> for $t<$case_0$(, $case)*, anyhow::Error> {
            fn from(value: anyhow::Error) -> Self {
                Self::$case_n(value)
            }
        }
        // NOTE: `.context` for `EitherN<…, anyhow::Error>`.
        impl<$case_0$(, $case)*> Context for $t<$case_0$(, $case)*, anyhow::Error> {
            fn context<C>(self, context: C) -> Self
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

pub trait Context {
    fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static;
}

impl<T, E: Context> Context for Result<T, E> {
    fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(err.context(context)),
        }
    }
}
