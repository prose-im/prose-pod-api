// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod cache;
mod concurrent_task_runner;
mod debounced_notify;
pub mod deserializers;
mod detect_mime_type;
pub mod either;
pub mod paginate;
pub mod sea_orm;
mod unaccent;

use crate::models::jid::{BareJid, NodeRef};

pub use self::cache::*;
pub use self::concurrent_task_runner::*;
pub use self::debounced_notify::*;
pub use self::deserializers::deserialize_null_as_some_none;
pub use self::detect_mime_type::*;
pub use self::unaccent::*;

pub trait JidExt {
    fn expect_username(&self) -> &NodeRef;
}

impl JidExt for BareJid {
    fn expect_username(&self) -> &NodeRef {
        self.node().expect("User JIDs should have a localpart")
    }
}

/// [`panic!`] in debug mode, [`tracing::error!`] in release.
#[inline(always)]
pub fn debug_panic_or_log_error(msg: String) {
    if cfg!(debug_assertions) {
        panic!("{msg}");
    } else {
        tracing::error!(msg);
    }
}

/// [`panic!`] in debug mode, [`tracing::warn!`] in release.
#[inline(always)]
pub fn debug_panic_or_log_warning(msg: String) {
    if cfg!(debug_assertions) {
        panic!("{msg}");
    } else {
        tracing::warn!(msg);
    }
}

#[macro_export]
macro_rules! wrapper_type {
    ($wrapper:ident, $t:ty $([+$option:ident])* $(; $derive:path)*) => {
        #[derive(Clone, PartialEq, Eq, Hash)]
        #[derive(serde_with::SerializeDisplay)]
        $(#[derive($derive)])*
        #[repr(transparent)]
        pub struct $wrapper($t);

        impl std::ops::Deref for $wrapper {
            type Target = $t;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $wrapper {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl std::fmt::Debug for $wrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl std::fmt::Display for $wrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl From<$t> for $wrapper {
            fn from(inner: $t) -> Self {
                Self(inner)
            }
        }

        impl Into<$t> for $wrapper {
            fn into(self) -> $t {
                self.0
            }
        }

        $(crate::wrapper_type!(__impls $wrapper, $t [+$option]);)*
    };
    (__impls $wrapper:ident, $t:ty [+FromStr]) => {
        impl std::str::FromStr for $wrapper {
            type Err = <$t as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$t>::from_str(s).map(Self)
            }
        }
    };
}

/// Generates a random string.
///
/// WARN: Do not generate secrets with this function! Instead, use
///   [`crate::auth::util::random_secret`].
#[must_use]
#[inline]
pub fn random_string_alphanumeric(length: usize) -> String {
    use rand::{distributions::Alphanumeric, Rng as _};

    // NOTE: Code taken from <https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters>.
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
}
