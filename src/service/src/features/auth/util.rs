// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use secrecy::SecretString;

/// Generates a random secret string.
#[inline]
pub fn random_secret(length: usize) -> SecretString {
    assert!(length >= 16);

    crate::util::random_string_alphanumeric(length).into()
}

/// Generates a random secret string (URL-safe).
#[inline]
pub fn random_secret_url_safe(length: usize) -> SecretString {
    // NOTE: Already URL-safe.
    random_secret(length)
}
