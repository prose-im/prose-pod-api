// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rand::{distributions::Alphanumeric, thread_rng, Rng as _};
use secrecy::SecretString;

/// Generates a random secret string.
#[inline]
pub fn random_secret(length: usize) -> SecretString {
    assert!(length >= 16);

    // NOTE: Code taken from <https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters>.
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
        .into()
}

/// Generates a random secret string (URL-safe).
#[inline]
pub fn random_secret_url_safe(length: usize) -> SecretString {
    // NOTE: Already URL-safe.
    random_secret(length)
}
