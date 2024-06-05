// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Cow;

use base64::{engine::general_purpose, DecodeError, Engine as _};
use serde::{Deserialize, Serialize};
use xmpp_parsers::sha1::{Digest, Sha1};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct ImageId(String);

impl AsRef<str> for ImageId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::str::FromStr for ImageId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ImageId(s.to_string()))
    }
}

impl<T> From<T> for ImageId
where
    T: Into<String>,
{
    fn from(s: T) -> ImageId {
        ImageId(s.into())
    }
}

impl std::fmt::Display for ImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum AvatarData {
    Base64(String),
    Data(Vec<u8>),
}

impl AvatarData {
    pub fn data(&self) -> std::result::Result<Cow<Vec<u8>>, DecodeError> {
        match self {
            AvatarData::Base64(base64) => Ok(Cow::Owned(general_purpose::STANDARD.decode(base64)?)),
            AvatarData::Data(data) => Ok(Cow::Borrowed(data)),
        }
    }

    pub fn base64(&self) -> Cow<str> {
        match self {
            AvatarData::Base64(base64) => Cow::Borrowed(base64),
            AvatarData::Data(data) => Cow::Owned(general_purpose::STANDARD.encode(data)),
        }
    }

    pub fn generate_sha1_checksum(&self) -> std::result::Result<ImageId, DecodeError> {
        let mut hasher = Sha1::new();
        hasher.update(self.data()?.as_ref());
        Ok(format!("{:x}", hasher.finalize()).into())
    }
}
