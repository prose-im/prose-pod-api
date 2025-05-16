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
use serde::{Deserialize, Serialize};
use service::auth::errors::InvalidCredentials;
use service::auth::{AuthService, Credentials, IsAdmin, UserInfo};
use service::factory_reset::factory_reset_controller;
use service::models::SerializableSecretString;
use service::secrets::SecretsStore;
use service::xmpp::ServerCtl;
use tracing::info;

use crate::error::Error;
use crate::AppState;

pub(super) fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", delete(factory_reset_route))
        .route_layer(from_extractor_with_state::<IsAdmin, _>(app_state.clone()))
        .with_state(app_state)
}

#[derive(Debug, Serialize, Deserialize)]
struct FactoryResetConfirmation {
    pub confirmation: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum FactoryResetRequest {
    PasswordConfirmation { password: SerializableSecretString },
    ConfirmationToken(FactoryResetConfirmation),
}

async fn factory_reset_route(
    State(AppState {
        db,
        app_config,
        lifecycle_manager,
        ..
    }): State<AppState>,
    server_ctl: ServerCtl,
    secrets_store: SecretsStore,
    user_info: UserInfo,
    auth_service: AuthService,
    Json(req): Json<FactoryResetRequest>,
) -> Result<Either<(StatusCode, Json<FactoryResetConfirmation>), StatusCode>, Error> {
    match req {
        FactoryResetRequest::PasswordConfirmation { password } => {
            let credentials = Credentials {
                jid: user_info.jid,
                password: password.into_secret_string(),
            };
            match factory_reset_controller::get_confirmation_code(&credentials, &auth_service).await
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
                &server_ctl,
                &app_config,
                &secrets_store,
            )
            .await?;

            info!("Restarting the API…");
            lifecycle_manager.set_restarting();

            Ok(Either::E2(StatusCode::RESET_CONTENT))
        }
    }
}

pub async fn restart_guard(
    State(AppState {
        lifecycle_manager, ..
    }): State<AppState>,
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
