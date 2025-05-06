// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::instrument;

use crate::util::Either;

use super::{errors::InvalidCredentials, AuthService, AuthToken, Credentials};

/// Log user in and return an authentication token.
#[instrument(
    name = "auth_controller::log_in",
    level = "trace",
    skip_all, fields(jid = credentials.jid.to_string()),
)]
pub async fn log_in(
    credentials: &Credentials,
    auth_service: &AuthService,
) -> Result<AuthToken, Either<InvalidCredentials, anyhow::Error>> {
    auth_service
        .log_in(&credentials.jid, &credentials.password)
        .await
}
