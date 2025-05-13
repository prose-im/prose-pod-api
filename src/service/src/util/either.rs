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
