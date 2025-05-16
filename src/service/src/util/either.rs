// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display};

macro_rules! gen {
    ($t:ident<$case1:ident$(, $case:ident)+>) => {
        pub enum $t<$case1$(, $case)+> {
            $case1($case1),
            $($case($case),)+
        }

        impl<$case1: Debug$(, $case: Debug)+> Debug for $t<$case1$(, $case)+> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::$case1(v) => Debug::fmt(&v, f),
                    $(Self::$case(v) => Debug::fmt(&v, f),)+
                }
            }
        }

        impl<$case1: Display$(, $case: Display)+> Display for $t<$case1$(, $case)+> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::$case1(v) => Display::fmt(&v, f),
                    $(Self::$case(v) => Display::fmt(&v, f),)+
                }
            }
        }
    };
}

gen!(Either<E1, E2>);
gen!(Either3<E1, E2, E3>);

// MARK: USEFUL (OPINIONATED) IMPLEMENTATIONS

pub trait Context {
    fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static;
}

impl<E1> From<anyhow::Error> for Either<E1, anyhow::Error> {
    fn from(value: anyhow::Error) -> Self {
        Self::E2(value)
    }
}
impl<E1> Context for Either<E1, anyhow::Error> {
    fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        match self {
            Self::E2(err) => Self::E2(err.context(context)),
            err => err,
        }
    }
}
impl<E1> From<sea_orm::DbErr> for Either<E1, anyhow::Error> {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::E2(anyhow::Error::new(value).context("Database error"))
    }
}

impl<E1, E2> From<anyhow::Error> for Either3<E1, E2, anyhow::Error> {
    fn from(value: anyhow::Error) -> Self {
        Self::E3(value)
    }
}
impl<E1, E2> Context for Either3<E1, E2, anyhow::Error> {
    fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        match self {
            Self::E3(err) => Self::E3(err.context(context)),
            err => err,
        }
    }
}
impl<E1, E2> From<sea_orm::DbErr> for Either3<E1, E2, anyhow::Error> {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::E3(anyhow::Error::new(value).context("Database error"))
    }
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
