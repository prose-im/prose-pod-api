// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod cache;
mod concurrent_task_runner;
mod debounced_notify;
mod deserializers;
mod detect_mime_type;
pub mod either;
mod sea_orm;
mod unaccent;

use crate::models::jid::{BareJid, DomainRef, NodeRef};

pub use self::cache::*;
pub use self::concurrent_task_runner::*;
pub use self::debounced_notify::*;
pub use self::deserializers::*;
pub use self::detect_mime_type::*;
pub use self::unaccent::*;

pub fn bare_jid_from_username(username: &NodeRef, server_domain: &DomainRef) -> BareJid {
    BareJid::from_parts(Some(username), server_domain)
}

#[macro_export]
macro_rules! wrapper_type {
    ($wrapper:ident, $t:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
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

        impl std::fmt::Display for $wrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl std::str::FromStr for $wrapper {
            type Err = <$t as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$t>::from_str(s).map(Self)
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
