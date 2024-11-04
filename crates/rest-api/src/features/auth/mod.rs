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

    use crate::error::prelude::*;

    impl_into_error!(
        AuthBasicError,
        ErrorCode::UNAUTHORIZED,
        vec![(
            "WWW-Authenticate".into(),
            r#"Basic realm="Admin only area", charset="UTF-8""#.into(),
        )]
    );
}
