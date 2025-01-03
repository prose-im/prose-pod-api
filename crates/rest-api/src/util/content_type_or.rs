// Copyright © davidpdrsn (https://gist.github.com/davidpdrsn/eb4e703e7e068ece3efd975b8f6bc340)

use std::marker::PhantomData;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::CONTENT_TYPE, request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::handler::HandlerCallWithExtractors;

// --- content types ----

macro_rules! impl_content_type {
    ($ident:ident, $mime:literal) => {
        #[derive(Copy, Clone)]
        pub struct $ident;

        impl ContentType for $ident {
            const NAME: &'static str = $mime;
        }
    };
}

impl_content_type!(ApplictionJson, "application/json");
impl_content_type!(TextPlain, "text/plain");
impl_content_type!(TextEventStream, "text/event-stream");

// --- implementation details ---

pub trait ContentType {
    const NAME: &'static str;
}

pub fn with_content_type<C, H>(handler: H) -> WithContentTypeHandler<C, H>
where
    C: ContentType,
{
    WithContentTypeHandler {
        content_type: PhantomData,
        handler,
    }
}

pub struct WithContentTypeHandler<C, H> {
    content_type: PhantomData<C>,
    handler: H,
}

impl<C, H> Clone for WithContentTypeHandler<C, H>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            content_type: self.content_type,
            handler: self.handler.clone(),
        }
    }
}

impl<C, H, T, S> HandlerCallWithExtractors<(WithContentType<C>, T), S>
    for WithContentTypeHandler<C, H>
where
    C: ContentType,
    H: HandlerCallWithExtractors<T, S>,
{
    type Future = H::Future;

    fn call(self, (_, extractors): (WithContentType<C>, T), state: S) -> Self::Future {
        self.handler.call(extractors, state)
    }
}

pub struct WithContentType<C>(PhantomData<C>);

#[async_trait]
impl<S, C> FromRequestParts<S> for WithContentType<C>
where
    C: ContentType,
    S: Send + Sync,
{
    type Rejection = WrongContentType;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let has_right_content_type = parts
            .headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map_or(false, |value| value.starts_with(C::NAME));

        if has_right_content_type {
            Ok(Self(PhantomData))
        } else {
            Err(WrongContentType)
        }
    }
}

pub struct WrongContentType;

impl IntoResponse for WrongContentType {
    fn into_response(self) -> Response {
        StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response()
    }
}
