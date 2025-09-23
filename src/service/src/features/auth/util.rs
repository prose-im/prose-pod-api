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

#[must_use]
#[inline]
pub fn random_oauth2_registration_key() -> SecretString {
    use rand::RngCore as _;

    let mut key = [0u8; 256];
    rand::thread_rng().fill_bytes(&mut key);

    // fn bytes_to_hex(bytes: &[u8]) -> String {
    //     bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
    // }
    fn bytes_to_base64(bytes: &[u8]) -> String {
        use base64::Engine as _;
        base64::prelude::BASE64_STANDARD.encode(bytes)
    }

    SecretString::from(bytes_to_base64(&key))
}
