// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod guards;
mod login;

pub use login::*;

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![login_route]
}

mod error {
    use http_auth_basic::AuthBasicError;
    use service::{
        auth::{auth_service, jwt_service},
        prosody::ProsodyOAuth2Error,
    };

    use crate::error::prelude::*;

    impl_into_error!(
        AuthBasicError,
        ErrorCode::UNAUTHORIZED,
        vec![(
            "WWW-Authenticate".into(),
            r#"Basic realm="Admin only area", charset="UTF-8""#.into(),
        )]
    );

    impl CustomErrorCode for jwt_service::Error {
        fn error_code(&self) -> ErrorCode {
            match self {
                Self::Sign(_) | Self::Other(_) => ErrorCode::INTERNAL_SERVER_ERROR,
                Self::Verify(_) | Self::InvalidClaim(_) => ErrorCode::UNAUTHORIZED,
            }
        }
    }
    impl_into_error!(jwt_service::Error);

    impl CustomErrorCode for auth_service::Error {
        fn error_code(&self) -> ErrorCode {
            match self {
                Self::InvalidCredentials
                | Self::ProsodyOAuth2Err(ProsodyOAuth2Error::Unauthorized(_)) => {
                    ErrorCode::UNAUTHORIZED
                }
                Self::ProsodyOAuth2Err(ProsodyOAuth2Error::Forbidden(_)) => ErrorCode::FORBIDDEN,
                _ => ErrorCode::INTERNAL_SERVER_ERROR,
            }
        }
    }
    impl_into_error!(auth_service::Error);
}
