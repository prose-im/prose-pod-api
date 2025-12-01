// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod errors;
pub mod extractors;
mod routes;

use axum::{middleware::from_extractor_with_state, routing::*};
use service::auth::{Authenticated, IsAdmin};

use crate::AppState;

pub use self::routes::*;

use super::members::MEMBER_ROUTE;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .nest(
            MEMBER_ROUTE,
            axum::Router::new()
                .route("/role", put(set_member_role_route))
                .route("/password", delete(request_password_reset_route))
                .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
                .route(
                    "/recovery-email-address",
                    MethodRouter::new()
                        .get(get_member_recovery_email_address_route)
                        .put(set_member_recovery_email_address_route),
                )
                .route_layer(from_extractor_with_state::<Authenticated, _>(
                    app_state.clone(),
                )),
        )
        .route("/v1/login", post(login_route))
        .route(
            "/v1/password-reset-tokens/{token}/use",
            put(reset_password_route),
        )
        .with_state(app_state)
}

pub mod models {
    use std::ops::Deref;

    use secrecy::SecretString;
    use serdev::{Deserialize, Serialize};
    use service::models::SerializableSecretString;

    // NOTE: We shouldn only validate password length during
    //   account creation and password reset. If we did it when
    //   parsing, we could prevent someone from logging in if they
    //   have chosen a shorter password using another tool.
    #[derive(Debug, Clone)]
    #[derive(Serialize, Deserialize)]
    #[repr(transparent)]
    pub struct Password(SerializableSecretString);

    impl Deref for Password {
        type Target = SerializableSecretString;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> From<T> for Password
    where
        SerializableSecretString: From<T>,
    {
        fn from(value: T) -> Self {
            Self(SerializableSecretString::from(value))
        }
    }
    impl Into<SecretString> for Password {
        fn into(self) -> SecretString {
            self.0.into()
        }
    }
}
