// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Context as _;
use jid::BareJid;
use secrecy::{ExposeSecret, SecretString};
use tracing::debug;

use crate::{
    auth::{errors::InvalidCredentials, AuthService},
    sea_orm::DatabaseConnection,
    secrets::{SecretsStore, ServiceAccountSecrets},
    server_config,
    util::either::Either,
    AppConfig,
};

use super::{server_ctl, ServerCtl};

/// Generates a very strong random password.
fn strong_random_password() -> SecretString {
    // 256 characters because why not
    crate::auth::util::strong_random_password(256)
}

async fn set_api_xmpp_password(
    server_ctl: &ServerCtl,
    app_config: &AppConfig,
    secrets_store: &SecretsStore,
    password: SecretString,
) -> anyhow::Result<()> {
    let api_jid = app_config.api_jid();

    (server_ctl.set_user_password(&api_jid, &password).await).context("ServerCtl error")?;
    secrets_store.set_prose_pod_api_xmpp_password(password);

    Ok(())
}

pub async fn reset_server_config(
    db: &DatabaseConnection,
    server_ctl: &ServerCtl,
    app_config: &AppConfig,
    secrets_store: &SecretsStore,
) -> anyhow::Result<()> {
    server_config::reset(db).await?;

    // Write the bootstrap configuration.
    let password = self::strong_random_password();
    server_ctl.reset_config(&password).await?;

    // Update the API user password to match the new one specified in the bootstrap configuration.
    self::set_api_xmpp_password(server_ctl, app_config, secrets_store, password.clone()).await?;

    // Store the new password in the environment variables to the next API instance
    // can access it (the screts store will be dropped efore next run).
    std::env::set_var(
        "PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD",
        password.expose_secret(),
    );

    // Apply the bootstrap configuration.
    server_ctl.reload().await?;

    Ok(())
}

pub async fn rotate_api_xmpp_password(
    server_ctl: &ServerCtl,
    app_config: &AppConfig,
    secrets_store: &SecretsStore,
) -> anyhow::Result<()> {
    self::set_api_xmpp_password(
        server_ctl,
        app_config,
        secrets_store,
        self::strong_random_password(),
    )
    .await
}

pub async fn create_service_accounts(
    server_ctl: &ServerCtl,
    app_config: &AppConfig,
    auth_service: &AuthService,
    secrets_store: &SecretsStore,
) -> Result<(), CreateServiceAccountError> {
    // NOTE: No need to create Prose Pod API's XMPP account as it's already created
    //   automatically when the XMPP server starts (using `mod_init_admin` in Prosody).

    // Create workspace XMPP account
    self::create_service_account(
        app_config.workspace_jid(),
        server_ctl,
        auth_service,
        secrets_store,
    )
    .await?;

    Ok(())
}

async fn create_service_account(
    jid: BareJid,
    server_ctl: &ServerCtl,
    auth_service: &AuthService,
    secrets_store: &SecretsStore,
) -> Result<(), CreateServiceAccountError> {
    debug!("Creating service account '{jid}'…");

    // Create the XMPP user account
    let password = self::strong_random_password();
    server_ctl.add_user(&jid, &password).await?;

    // Log in as the service account (to get a JWT with access tokens)
    let auth_token = auth_service.log_in(&jid, &password).await?;

    // Store the secrets
    let secrets = ServiceAccountSecrets {
        password,
        prosody_token: auth_token.clone(),
    };
    secrets_store.set_service_account_secrets(jid, secrets);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateServiceAccountError {
    #[error("Could not create XMPP account: {0}")]
    CouldNotCreateXmppAccount(#[from] server_ctl::Error),
    #[error("Could not log in: {0}")]
    CouldNotLogIn(Either<InvalidCredentials, anyhow::Error>),
}

impl From<Either<InvalidCredentials, anyhow::Error>> for CreateServiceAccountError {
    fn from(error: Either<InvalidCredentials, anyhow::Error>) -> Self {
        Self::CouldNotLogIn(error)
    }
}
