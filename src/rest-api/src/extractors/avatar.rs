// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    body::Bytes,
    extract::rejection::{BytesRejection, JsonRejection, StringRejection},
    response::IntoResponse,
    Json,
};
use axum_extra::headers::{ContentType, HeaderMapExt};
use service::models::{Avatar, AvatarDecodeError};

use crate::error::{ErrorCode, HttpApiError};

use super::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum AvatarFromRequestError {
    #[error("Invalid bytes: {0}")]
    InvalidBytes(#[from] BytesRejection),
    #[error("Invalid string: {0}")]
    InvalidString(#[from] StringRejection),
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] JsonRejection),
    #[error("Invalid avatar: {0}")]
    InvalidAvatar(#[from] AvatarDecodeError),
    #[error("Unsupported media type.")]
    UnsupportedMediaType,
}

impl HttpApiError for AvatarFromRequestError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::UnsupportedMediaType
            | Self::InvalidAvatar(AvatarDecodeError::UnsupportedMediaType) => {
                ErrorCode::UNSUPPORTED_MEDIA_TYPE
            }
            Self::InvalidBytes(_)
            | Self::InvalidString(_)
            | Self::InvalidJson(_)
            | Self::InvalidAvatar(_) => ErrorCode::BAD_REQUEST,
        }
    }
}

impl FromRequest<AppState> for Avatar {
    type Rejection = AvatarFromRequestError;

    async fn from_request(req: Request, _state: &AppState) -> Result<Self, Self::Rejection> {
        let content_type = req.headers().typed_get::<ContentType>();

        async fn from_bytes(req: Request) -> Result<Avatar, AvatarFromRequestError> {
            let bytes = Bytes::from_request(req, &()).await?;
            // TODO: Find a way to avoid this copy? Using `bytes::Bytes`
            //   internally could be a good idea.
            let avatar = Avatar::try_from_bytes(bytes)?;
            Ok(avatar)
        }

        async fn from_text(req: Request) -> Result<Avatar, AvatarFromRequestError> {
            let string = String::from_request(req, &()).await?;
            let avatar = Avatar::try_from_base64(string)?;
            Ok(avatar)
        }

        async fn from_json(req: Request) -> Result<Avatar, AvatarFromRequestError> {
            let Json(string) = Json::<String>::from_request(req, &()).await?;
            let avatar = Avatar::try_from_base64(string)?;
            Ok(avatar)
        }

        match content_type {
            None => from_bytes(req).await,
            Some(ct) if ct == ContentType::octet_stream() => from_bytes(req).await,
            Some(ct) if ct.to_string().starts_with("image/") => from_bytes(req).await,
            Some(ct) if ct == ContentType::text() => from_text(req).await,
            Some(ct) if ct == ContentType::json() => from_json(req).await,
            _ => Err(Self::Rejection::UnsupportedMediaType),
        }
    }
}

// MARK: - Boilerplate

impl IntoResponse for AvatarFromRequestError {
    fn into_response(self) -> axum::response::Response {
        Error::from(self).into_response()
    }
}
