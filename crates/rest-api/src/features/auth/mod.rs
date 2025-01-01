// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod guards;
mod login;

use axum::routing::post;

pub use self::login::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    rocket::routes![login_route]
}

pub(super) fn router<S: crate::AxumState>() -> axum::Router<S> {
    axum::Router::new().route("/v1/login", post(login_route_axum))
}

mod error {
    use http_auth_basic::AuthBasicError;
    use service::{auth::auth_service, prosody::ProsodyOAuth2Error};

    use crate::error::prelude::*;

    impl_into_error!(
        AuthBasicError,
        ErrorCode::UNAUTHORIZED,
        vec![(
            "WWW-Authenticate".into(),
            r#"Basic realm="Admin only area", charset="UTF-8""#.into(),
        )]
    );

    impl HttpApiError for auth_service::Error {
        fn code(&self) -> ErrorCode {
            match self {
                Self::InvalidCredentials => ErrorCode::UNAUTHORIZED,
                Self::ProsodyOAuth2Err(err) => err.code(),
                _ => ErrorCode::INTERNAL_SERVER_ERROR,
            }
        }

        fn message(&self) -> String {
            std::format!("auth_service::Error: {self}")
        }

        fn debug_info(&self) -> Option<serde_json::Value> {
            match self {
                Self::ProsodyOAuth2Err(err) => err.debug_info(),
                _ => None,
            }
        }
    }

    impl HttpApiError for ProsodyOAuth2Error {
        fn code(&self) -> ErrorCode {
            match self {
                Self::Unauthorized(_) => ErrorCode::UNAUTHORIZED,
                Self::Forbidden(_) => ErrorCode::FORBIDDEN,
                _ => ErrorCode::INTERNAL_SERVER_ERROR,
            }
        }

        fn message(&self) -> String {
            std::format!("ProsodyOAuth2Error: {self}")
        }

        fn debug_info(&self) -> Option<serde_json::Value> {
            match self {
                Self::UnexpectedResponse(err) => err.debug_info(),
                _ => None,
            }
        }
    }
}
