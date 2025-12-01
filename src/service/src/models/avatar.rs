// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use bytes::Bytes;
use mime::Mime;
use serde_with::{base64::Base64, serde_as, DisplayFromStr};
use serdev::Serialize;
use validator::Validate;

use crate::{
    models::BytesAmount,
    util::{detect_image_media_type, SUPPORTED_IMAGE_MEDIA_TYPES},
};

// NOTE: This is the very maximum the Pod API will accept. While a softer limit
//   could be configured via the app configuration (checked when uploading), this
//   limit ensures no [`Avatar`] value can ever exceed 10MB (to prevent abuse).
pub(crate) const AVATAR_MAX_LENGTH: BytesAmount = BytesAmount::MegaBytes(10);

// TODO: Make max size 512kB (decoded bytes) via app configuration.
/// Avatar, decoded from raw bytes or Base64.
///
/// Media type detected from magic bytes for improved security.
#[derive(Clone, PartialEq, Eq)]
#[serde_as]
#[derive(Serialize, Validate, serdev::Deserialize)]
#[serde(validate = "Self::validate")]
pub struct Avatar {
    #[serde(rename = "base64")]
    #[serde_as(as = "Base64")]
    bytes: Bytes,

    #[serde(rename = "type")]
    #[serde_as(as = "DisplayFromStr")]
    media_type: Mime,
}

#[derive(Debug, thiserror::Error)]
pub enum AvatarDecodeError {
    #[error("Avatar too large. Max length: {AVATAR_MAX_LENGTH}.")]
    TooLarge,
    #[error("Unsupported media type. Supported: {SUPPORTED_IMAGE_MEDIA_TYPES:?}.")]
    UnsupportedMediaType,
    #[error("Invalid Base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
}

impl Avatar {
    pub fn try_from_bytes(bytes: Bytes) -> Result<Self, AvatarDecodeError> {
        if bytes.len() > AVATAR_MAX_LENGTH.as_bytes() as usize {
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
    pub fn try_from_base64(bytes: impl AsRef<[u8]>) -> Result<Self, AvatarDecodeError> {
        use base64::{prelude::BASE64_STANDARD, Engine as _};

        let bytes = BASE64_STANDARD
            .decode(bytes)
            .map_err(AvatarDecodeError::InvalidBase64)?;

        Self::try_from_bytes(Bytes::from(bytes))
    }
}

impl std::fmt::Debug for Avatar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(
            &format!("Avatar({type}; {len}B)", type = self.media_type, len = self.bytes.len()),
            f,
        )
    }
}

impl Avatar {
    fn validate(&self) -> Result<(), AvatarDecodeError> {
        if self.bytes.len() <= AVATAR_MAX_LENGTH.as_bytes() as usize {
            Ok(())
        } else {
            Err(AvatarDecodeError::TooLarge)
        }
    }
}

// MARK: - Boilerplate

impl Avatar {
    #[inline]
    pub fn into_inner(self) -> Bytes {
        self.bytes
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

impl AsRef<[u8]> for Avatar {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl std::ops::Deref for Avatar {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}
