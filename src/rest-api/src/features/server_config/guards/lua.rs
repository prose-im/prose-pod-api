// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::error::{Error, ErrorCode, HttpApiError, LogLevel};
use axum::extract::{rejection::*, FromRequest, OptionalFromRequest, Request};
use axum::http::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode,
};
use axum::response::{IntoResponse, Response};

#[derive(Debug, Clone, Default)]
pub struct Lua(pub String);

impl<S> FromRequest<S> for Lua
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if lua_content_type(req.headers()) {
            let content = (String::from_request(req, state).await).map_err(LuaRejection::from)?;
            Ok(Self(content))
        } else {
            Err(Error::from(LuaRejection::MissingLuaContentType))
        }
    }
}

impl<S> OptionalFromRequest<S> for Lua
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
        let headers = req.headers();
        if headers.get(header::CONTENT_TYPE).is_some() {
            if lua_content_type(headers) {
                let content =
                    (String::from_request(req, state).await).map_err(LuaRejection::from)?;
                Ok(Some(Self(content)))
            } else {
                Err(Error::from(LuaRejection::MissingLuaContentType))
            }
        } else {
            Ok(None)
        }
    }
}

fn lua_content_type(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return false;
    };

    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    let Ok(mime) = content_type.parse::<mime::Mime>() else {
        return false;
    };

    let is_lua_content_type = (mime.type_() == "application" && mime.subtype() == "json")
        || (mime.type_() == "text" && mime.subtype() == "x-lua");

    is_lua_content_type
}

impl IntoResponse for Lua {
    fn into_response(self) -> Response {
        (
            [(header::CONTENT_TYPE, HeaderValue::from_static("text/x-lua"))],
            self.0,
        )
            .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LuaRejection {
    #[error("Expected request with `Content-Type: text/x-lua`")]
    MissingLuaContentType,
    #[error("{0}")]
    StringRejection(#[from] StringRejection),
}

impl HttpApiError for LuaRejection {
    fn code(&self) -> ErrorCode {
        ErrorCode {
            value: "unsupported_media_type",
            http_status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            log_level: LogLevel::Info,
        }
    }
}
