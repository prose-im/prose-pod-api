// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use secrecy::SecretString;
use serdev::{Deserialize, Serialize, Serializer};

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[repr(transparent)]
pub struct SerializableSecretString(#[serde(serialize_with = "expose_secret")] SecretString);

impl SerializableSecretString {
    pub fn len(&self) -> usize {
        secrecy::ExposeSecret::expose_secret(&self.0).len()
    }
}

impl std::ops::Deref for SerializableSecretString {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for SerializableSecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<secret>")
    }
}

fn expose_secret<'a, S>(secret: &'a SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(secrecy::ExposeSecret::expose_secret(secret))
}

impl<T> From<T> for SerializableSecretString
where
    SecretString: From<T>,
{
    fn from(value: T) -> Self {
        Self(SecretString::from(value))
    }
}

impl Into<SecretString> for SerializableSecretString {
    fn into(self) -> SecretString {
        self.0
    }
}
