// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use secrecy::{ExposeSecret as _, SecretString, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SerializableSecretString(String);
impl Zeroize for SerializableSecretString {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
impl SerializableSecret for SerializableSecretString {}
impl From<SecretString> for SerializableSecretString {
    fn from(value: SecretString) -> Self {
        Self(value.expose_secret().to_owned())
    }
}
impl SerializableSecretString {
    pub fn into_secret_string(self) -> SecretString {
        SecretString::new(self.0)
    }
}
impl Into<SecretString> for SerializableSecretString {
    fn into(self) -> SecretString {
        self.into_secret_string()
    }
}
