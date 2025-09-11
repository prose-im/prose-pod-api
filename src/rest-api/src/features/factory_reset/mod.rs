// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Cow;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::{from_extractor_with_state, Next};
use axum::response::Response;
use axum::routing::*;
use axum::Json;
use axum_extra::either::Either;
use serdev::Serialize;
use service::auth::errors::InvalidCredentials;
use service::auth::{AuthService, Credentials, IsAdmin, UserInfo};
use service::factory_reset::factory_reset_controller::{
    self, FACTORY_RESET_CONFIRMATION_CODE_LENGTH,
};
use service::secrets::SecretsStore;
use service::xmpp::ServerCtl;
use tracing::info;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::error::Error;
use crate::features::auth::models::Password;
use crate::{AppState, MinimalAppState};

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", delete(factory_reset_route))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

#[derive(Debug)]
#[derive(Serialize, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
struct FactoryResetConfirmation {
    pub confirmation: String,
}

#[derive(Debug)]
#[derive(serdev::Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
enum FactoryResetRequest {
    PasswordConfirmation { password: Password },
    ConfirmationToken(FactoryResetConfirmation),
}

async fn factory_reset_route(
    State(MinimalAppState {
        ref lifecycle_manager,
        ..
    }): State<MinimalAppState>,
    State(AppState {
        db, ref app_config, ..
    }): State<AppState>,
    ref server_ctl: ServerCtl,
    ref secrets_store: SecretsStore,
    user_info: UserInfo,
    ref auth_service: AuthService,
    Json(req): Json<FactoryResetRequest>,
) -> Result<Either<(StatusCode, Json<FactoryResetConfirmation>), StatusCode>, Error> {
    match req {
        FactoryResetRequest::PasswordConfirmation { password } => {
            let credentials = Credentials {
                jid: user_info.jid,
                password: password.into(),
            };
            match factory_reset_controller::get_confirmation_code(&credentials, auth_service).await
            {
                Ok(confirmation) => Ok(Either::E1((
                    StatusCode::ACCEPTED,
                    Json(FactoryResetConfirmation { confirmation }),
                ))),
                Err(err @ InvalidCredentials) => Err(Error::from(err)),
            }
        }
        FactoryResetRequest::ConfirmationToken(FactoryResetConfirmation { confirmation }) => {
            factory_reset_controller::perform_factory_reset(
                confirmation,
                db,
                server_ctl,
                app_config,
                secrets_store,
            )
            .await?;

            info!("Restarting the API…");
            lifecycle_manager.set_restarting();

            Ok(Either::E2(StatusCode::RESET_CONTENT))
        }
    }
}

pub async fn restart_guard(
    State(MinimalAppState {
        lifecycle_manager, ..
    }): State<MinimalAppState>,
    request: Request,
    next: Next,
) -> Response {
    if lifecycle_manager.is_restarting() {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            // NOTE: A second should be enough, the API usually takes around 60ms to start.
            .header("Retry-After", 1)
            .body("The API is restarting.".into())
            .unwrap();
    }
    next.run(request).await
}

// MARK: Validation

impl Validate for FactoryResetConfirmation {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.confirmation.len() != FACTORY_RESET_CONFIRMATION_CODE_LENGTH {
            errors.add("confirmation", ValidationError::new("length").with_message(Cow::Owned(format!("Invalid confirmation code: Expected length is {FACTORY_RESET_CONFIRMATION_CODE_LENGTH}."))));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for FactoryResetRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Self::PasswordConfirmation { password } => password.validate(),
            Self::ConfirmationToken(confirmation) => confirmation.validate(),
        }
    }
}

// BOILERPLATE

mod error {
    use crate::error::prelude::*;

    impl HttpApiError for service::factory_reset::factory_reset_controller::InvalidConfirmationCode {
        fn code(&self) -> ErrorCode {
            ErrorCode {
                value: "invalid_confirmation_code",
                http_status: StatusCode::BAD_REQUEST,
                log_level: LogLevel::Info,
            }
        }
    }
}
