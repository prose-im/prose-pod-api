// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug, Clone)]
pub struct SecretString(secrecy::SecretString);

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret as _;
        self.0.expose_secret().eq(other.0.expose_secret())
    }
}

impl Eq for SecretString {}

impl std::ops::Deref for SecretString {
    type Target = secrecy::SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
