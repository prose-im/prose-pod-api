// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod errors;
pub mod extractors;
mod routes;

use axum::{middleware::from_extractor_with_state, routing::*};
use service::auth::IsAdmin;

use crate::AppState;

pub use self::routes::*;

use super::members::MEMBER_ROUTE;

// NOTE: At the moment we’re using “demo” as the demo password. I (@RemiBardon)
//   don’t want to have to fix all of that now so we’ll use 4 until I take the
//   time to regenerate all scenarios (or add a debug-only config to bypass it).
pub const MINIMUM_PASSWORD_LENGTH: usize = 4;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route(&format!("{MEMBER_ROUTE}/role"), put(set_member_role_route))
        .route(
            &format!("{MEMBER_ROUTE}/password"),
            delete(request_password_reset_route),
        )
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .route("/v1/login", post(login_route))
        .route(
            "/v1/password-reset-tokens/{token}/use",
            put(reset_password_route),
        )
        .with_state(app_state)
}

pub mod models {
    use std::{borrow::Cow, ops::Deref};

    use secrecy::SecretString;
    use serdev::Serialize;
    use service::models::SerializableSecretString;
    use validator::{Validate, ValidationError, ValidationErrors};

    use crate::features::auth::MINIMUM_PASSWORD_LENGTH;

    #[derive(Debug, Clone)]
    #[derive(Serialize, serdev::Deserialize)]
    #[serde(validate = "Validate::validate")]
    pub struct Password(SerializableSecretString);

    impl Deref for Password {
        type Target = SerializableSecretString;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Validate for Password {
        fn validate(&self) -> Result<(), ValidationErrors> {
            if self.0.len() < MINIMUM_PASSWORD_LENGTH {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "__self__",
                    ValidationError::new("password_too_short")
                        .with_message(Cow::Borrowed("Password too short.")),
                );
                Err(errors)
            } else {
                Ok(())
            }
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
