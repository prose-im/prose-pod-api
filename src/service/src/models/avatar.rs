// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{borrow::Cow, sync::Arc};

use mime::Mime;
use serde_with::{base64::Base64, serde_as, DisplayFromStr};
use serdev::{Serialize, Serializer};

use crate::util::{detect_image_media_type, SUPPORTED_IMAGE_MEDIA_TYPES};

/// 512kB
const AVATAR_MAX_LENGTH: usize = 512_000;

/// Avatar, decoded from raw bytes or Base64.
///
/// Media type detected from magic bytes for improved security.
///
/// NOTE: Max size is 512kB (decoded bytes).
#[derive(PartialEq, Eq)]
#[serde_as]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
pub struct Avatar<'a> {
    #[serde(rename = "base64")]
    #[serde_as(as = "Base64")]
    bytes: Cow<'a, [u8]>,
    #[serde(rename = "type")]
    #[serde_as(as = "DisplayFromStr")]
    media_type: Mime,
}

#[derive(Debug, thiserror::Error)]
pub enum AvatarDecodeError {
    #[error("Avatar too large. Max length: {AVATAR_MAX_LENGTH}B.")]
    TooLarge,
    #[error("Unsupported media type. Supported: {SUPPORTED_IMAGE_MEDIA_TYPES:?}.")]
    UnsupportedMediaType,
    #[error("Invalid Base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
}

impl<'a> Avatar<'a> {
    pub fn try_from_bytes(bytes: Cow<'a, [u8]>) -> Result<Self, AvatarDecodeError> {
        if bytes.len() > AVATAR_MAX_LENGTH {
            return Err(AvatarDecodeError::TooLarge);
        }

        let media_type =
            detect_image_media_type(&bytes).ok_or(AvatarDecodeError::UnsupportedMediaType)?;

        Ok(Self {
            bytes,
            media_type: media_type,
        })
    }

    #[inline]
    pub fn try_from_base64(bytes: Cow<'a, [u8]>) -> Result<Self, AvatarDecodeError> {
        use base64::{prelude::BASE64_STANDARD, Engine as _};

        let bytes = BASE64_STANDARD
            .decode(bytes)
            .map_err(AvatarDecodeError::InvalidBase64)?;

        Self::try_from_bytes(Cow::Owned(bytes))
    }

    #[inline]
    pub fn try_from_base64_str(str: &'a str) -> Result<Self, AvatarDecodeError> {
        Self::try_from_base64(Cow::Borrowed(str.as_bytes()))
    }

    #[inline]
    pub fn try_from_base64_string(string: String) -> Result<Self, AvatarDecodeError> {
        Self::try_from_base64(Cow::Owned(string.into_bytes()))
    }
}

impl<'a> std::fmt::Debug for Avatar<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(
            &format!("Avatar({type}; {len}B)", type = self.media_type, len = self.bytes.len()),
            f,
        )
    }
}

// MARK: - Owned variant

/// Owned variant of [`Avatar`].
#[derive(Clone, PartialEq, Eq)]
pub struct AvatarOwned {
    bytes: Arc<Vec<u8>>,
    media_type: Arc<Mime>,
}

impl AvatarOwned {
    pub fn as_avatar<'a>(&'a self) -> Avatar<'a> {
        Avatar {
            bytes: Cow::Borrowed(&self.bytes),
            media_type: self.media_type.as_ref().clone(),
        }
    }
}

impl<'a> std::borrow::Borrow<Avatar<'a>> for AvatarOwned {
    fn borrow(&self) -> &Avatar<'a> {
        unimplemented!("Use `.as_avatar()` instead.")
    }
}

impl<'a> ToOwned for Avatar<'a> {
    type Owned = AvatarOwned;

    fn to_owned(&self) -> Self::Owned {
        AvatarOwned {
            bytes: Arc::new(self.bytes.to_vec()),
            media_type: Arc::new(self.media_type.clone()),
        }
    }
}

impl std::fmt::Debug for AvatarOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(
            &format!("AvatarOwned({type}; {len}B)", type = self.media_type, len = self.bytes.len()),
            f,
        )
    }
}

// MARK: - Boilerplate

impl<'a> Avatar<'a> {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn media_type(&self) -> &Mime {
        &self.media_type
    }
}

impl AvatarOwned {
    #[inline]
    pub fn bytes(&self) -> Arc<Vec<u8>> {
        self.bytes.clone()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn media_type(&self) -> &Mime {
        &self.media_type
    }
}

impl<'a> TryFrom<Vec<u8>> for Avatar<'a> {
    type Error = AvatarDecodeError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_bytes(Cow::Owned(bytes))
    }
}

impl<'a> AsRef<[u8]> for Avatar<'a> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<[u8]> for AvatarOwned {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> std::ops::Deref for Avatar<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl Serialize for AvatarOwned {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_avatar().serialize(serializer)
    }
}
