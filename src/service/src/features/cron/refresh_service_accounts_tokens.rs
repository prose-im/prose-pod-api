// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::SystemTime;

use sea_orm::DatabaseConnection;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::*;

use crate::{
    auth::AuthService, secrets::SecretsStore, server_config::ServerConfigRepository, AppConfig,
};

#[derive(Debug, Clone)]
pub struct Context {
    pub app_config: AppConfig,
    pub db: DatabaseConnection,
    pub secrets_store: SecretsStore,
    pub auth_service: AuthService,
}

#[instrument(level = "trace", skip_all, ret(level = "warn"))]
pub async fn run(
    Context {
        ref app_config,
        ref db,
        ref secrets_store,
        ref auth_service,
    }: Context,
) {
    let oauth2_tokens_ttl = (app_config.auth.token_ttl.num_seconds())
        .expect("`app_config.auth.token_ttl` contains years or months. Not supported.");

    // If the TTL is `0` (which is the case in some tests), don’t run the task.
    if oauth2_tokens_ttl == 0. {
        return;
    }

    let refresh_interval = Duration::from_secs_f32(
        // NOTE: We cannot refresh tokens after the TTL as they would
        //   be expired for some time (leading to unexpected scenarios)
        //   therefore we set the refresh interval to $0.9 * TTL$.
        oauth2_tokens_ttl * 0.9,
    );

    let ticker_interval = Duration::from_secs(10);
    // NOTE: `tokio::time::interval` avoids time drifts.
    let mut ticker = interval(ticker_interval);
    // Configure ticker so it skips missed ticks if the API was suspended
    // (each token refresh would just cause a new token to be created).
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    // NOTE: `SystemTime::now()` reflects actual wall-clock time, even after a
    //   container pause.
    let mut last_refresh = SystemTime::now();

    info!(
        "Will refresh service accounts tokens in {secs}s",
        secs = refresh_interval.as_secs()
    );
    loop {
        ticker.tick().await;

        let Ok(lifetime) = SystemTime::now().duration_since(last_refresh) else {
            warn!("Time went backwards.");
            continue;
        };

        if lifetime < refresh_interval {
            trace!(
                "Service accounts tokens are still valid, will retry in {secs}s",
                secs = ticker_interval.as_secs(),
            );
            continue;
        }

        if lifetime.as_secs_f32() > oauth2_tokens_ttl {
            warn!(
                "Service accounts tokens were invalid for some time (TTL exceeded for {secs_since_expiry}s). This was likely caused by the API being suspended. Will refresh all tokens now to recover from this unexpected state.",
                secs_since_expiry = lifetime.as_secs_f32() - oauth2_tokens_ttl,
            );
        }

        info!("Refreshing service accounts tokens…");

        let server_config = match ServerConfigRepository::get(db).await {
            Ok(Some(server_config)) => server_config,
            Ok(None) => {
                info!("Not refreshing service accounts tokens: Server config not initialized.");
                continue;
            }
            Err(err) => {
                error!("Could not refresh service accounts tokens: {err}");
                continue;
            }
        };
        let domain = server_config.domain;

        let service_accounts = vec![app_config.workspace_jid(&domain)];

        for jid in service_accounts.iter() {
            let Some(password) = secrets_store.get_service_account_password(jid) else {
                error!(
                    "Could not refresh service accounts tokens: No password stored for '{jid}'."
                );
                continue;
            };
            let token = match auth_service.log_in(jid, &password).await {
                Ok(token) => token,
                Err(err) => {
                    error!("Could not refresh service accounts tokens: {err}");
                    continue;
                }
            };
            if let Err(err) = secrets_store.set_service_account_prosody_token(jid, token.0) {
                error!("Could not refresh service accounts tokens: {err}");
                continue;
            };
            info!("Refreshed token for service accounts '{jid}'…");
        }

        last_refresh = SystemTime::now();
        info!(
            "Will refresh service accounts tokens in {secs}s",
            secs = refresh_interval.as_secs()
        );
    }
}

// ===== BOILERPLATE =====

impl From<super::CronContext> for Context {
    fn from(value: super::CronContext) -> Self {
        Self {
            app_config: value.app_config,
            db: value.db,
            secrets_store: value.secrets_store,
            auth_service: value.auth_service,
        }
    }
}
