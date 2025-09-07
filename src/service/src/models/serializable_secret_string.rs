// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;

use secrecy::{zeroize::Zeroize, ExposeSecret, SecretString, SerializableSecret};
use serdev::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[repr(transparent)]
pub struct SerializableSecretString(String);
impl Zeroize for SerializableSecretString {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
impl SerializableSecret for SerializableSecretString {}
impl From<String> for SerializableSecretString {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl<'a> From<&'a str> for SerializableSecretString {
    fn from(value: &'a str) -> Self {
        Self(value.to_owned())
    }
}
impl From<SecretString> for SerializableSecretString {
    fn from(value: SecretString) -> Self {
        Self(value.expose_secret().to_owned())
    }
}
impl ExposeSecret<String> for SerializableSecretString {
    fn expose_secret(&self) -> &String {
        &self.0
    }
}
impl SerializableSecretString {
    pub fn into_secret_string(self) -> SecretString {
        SecretString::from(self.0)
    }
}
impl Into<SecretString> for SerializableSecretString {
    fn into(self) -> SecretString {
        self.into_secret_string()
    }
}
impl Debug for SerializableSecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<secret>")
    }
}
