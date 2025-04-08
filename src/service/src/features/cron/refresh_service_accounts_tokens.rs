// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::DatabaseConnection;
use tokio::time::{sleep, Duration};
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
    // If the TTL is `0` (which is the case in some tests), don’t run the task.
    if app_config.server.oauth2_access_token_ttl == 0 {
        return;
    }

    let refresh_interval = Duration::from_secs_f64({
        // NOTE: We cannot refresh tokens after the TTL as they would
        //   be expired for some time (leading to unexpected scenarios)
        //   therefore we set the refresh interval to $0.9 * TTL$.
        (app_config.server.oauth2_access_token_ttl as f64) * 0.9
    });

    loop {
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

        info!(
            "Will refresh service accounts tokens again in {secs}s",
            secs = refresh_interval.as_secs()
        );
        sleep(refresh_interval).await;
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
