// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use chrono::Utc;
use sea_orm::DatabaseConnection;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::*;

use crate::{
    auth::{password_reset_tokens, PasswordResetRecord},
    AppConfig,
};

#[derive(Debug)]
pub struct Context {
    pub app_config: Arc<AppConfig>,
    pub db: DatabaseConnection,
}

#[instrument(level = "trace", skip_all, ret(level = "warn"))]
pub async fn run(
    Context {
        ref app_config,
        ref db,
    }: Context,
) {
    let tokens_ttl = (app_config.auth.password_reset_token_ttl)
        .num_seconds()
        .expect(
            "`app_config.auth.password_reset_token_ttl` contains years or months. Not supported.",
        );

    // If the TTL is `0` (which is the case in some tests), don’t run the task.
    if tokens_ttl == 0. {
        info!("Not deleting expired password reset tokens: TTL is zero.");
        return;
    }

    let ticker_interval = Duration::from_secs_f32(tokens_ttl);
    // NOTE: `tokio::time::interval` avoids time drifts.
    let mut ticker = interval(ticker_interval);
    // Configure ticker so it skips missed ticks if the API was suspended
    // (each garbage collection would just cause database reads for nothing).
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    debug!(
        "Will delete expired password reset tokens in {secs}s",
        secs = ticker_interval.as_secs(),
    );
    loop {
        ticker.tick().await;

        debug!("Deleting expired password reset tokens…");

        let tokens = match password_reset_tokens::get_all(db).await {
            Ok(tokens) if tokens.is_empty() => continue,
            Ok(tokens) => tokens,
            Err(err) => {
                error!("Could not query expired password reset tokens: {err}");
                continue;
            }
        };

        for (key, value) in tokens {
            match serde_json::from_value::<PasswordResetRecord>(value) {
                Ok(record) => {
                    if Utc::now() > record.expires_at {
                        if let Err(err) = password_reset_tokens::kv_store_delete(db, &key).await {
                            error!("Could not delete expired password reset token record from database: {err}");
                        }
                    }
                }
                Err(err) => error!("Invalid password reset token record in database: {err}"),
            }
        }

        debug!(
            "Will delete expired password reset tokens in {secs}s",
            secs = ticker_interval.as_secs(),
        );
    }
}

// MARK: - Boilerplate

impl From<super::CronContext> for Context {
    fn from(value: super::CronContext) -> Self {
        Self {
            app_config: value.app_config,
            db: value.db,
        }
    }
}
