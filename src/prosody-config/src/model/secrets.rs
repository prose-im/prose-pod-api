// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;

use secrecy::zeroize::Zeroize;
#[cfg(feature = "serde")]
use secrecy::SerializableSecret;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct SecretString(String);

impl Zeroize for SecretString {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(feature = "serde")]
impl SerializableSecret for SecretString {}

impl From<secrecy::SecretString> for SecretString {
    fn from(value: secrecy::SecretString) -> Self {
        use secrecy::ExposeSecret as _;
        Self(value.expose_secret().to_owned())
    }
}

impl SecretString {
    pub fn into_secret_string(self) -> secrecy::SecretString {
        secrecy::SecretString::from(self.0)
    }
}

impl Into<secrecy::SecretString> for SecretString {
    fn into(self) -> secrecy::SecretString {
        self.into_secret_string()
    }
}

impl Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<secret>")
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for SecretString {}
