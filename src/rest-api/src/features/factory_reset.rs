// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
use service::factory_reset::{
    factory_reset_controller, FactoryResetConfirmationCode, FactoryResetService,
};
use service::secrets::SecretsStore;
use service::xmpp::ServerCtl;
use tracing::info;
use validator::{Validate, ValidationErrors};

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
#[derive(Serialize, Validate, serdev::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(validate = "Validate::validate")]
struct FactoryResetConfirmation {
    #[validate(nested)]
    pub confirmation: FactoryResetConfirmationCode,
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
    ref factory_reset_service: FactoryResetService,
    Json(req): Json<FactoryResetRequest>,
) -> Result<Either<(StatusCode, Json<FactoryResetConfirmation>), StatusCode>, Error> {
    match req {
        FactoryResetRequest::PasswordConfirmation { password } => {
            let ref credentials = Credentials {
                jid: user_info.jid,
                password: password.into(),
            };
            match factory_reset_controller::get_confirmation_code(
                factory_reset_service,
                credentials,
                auth_service,
            )
            .await
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
                factory_reset_service,
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

impl Validate for FactoryResetRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Self::PasswordConfirmation { password } => password.validate(),
            Self::ConfirmationToken(confirmation) => confirmation.validate(),
        }
    }
}

// MARK: - Extractors

mod extractors {
    use std::convert::Infallible;

    use axum::{extract::FromRequestParts, http::request::Parts};
    use service::factory_reset::FactoryResetService;

    use crate::AppState;

    impl FromRequestParts<AppState> for FactoryResetService {
        type Rejection = Infallible;

        #[tracing::instrument(
            name = "req::extract::factory_reset_service",
            level = "trace",
            skip_all,
            err
        )]
        async fn from_request_parts(
            _parts: &mut Parts,
            state: &AppState,
        ) -> Result<Self, Self::Rejection> {
            Ok(state.factory_reset_service.clone())
        }
    }
}

// MARK: - Boilerplate

mod errors {
    use crate::error::prelude::*;

    impl HttpApiError for service::factory_reset::InvalidConfirmationCode {
        fn code(&self) -> ErrorCode {
            ErrorCode {
                value: "invalid_confirmation_code",
                http_status: StatusCode::BAD_REQUEST,
                log_level: LogLevel::Info,
            }
        }
    }
}
