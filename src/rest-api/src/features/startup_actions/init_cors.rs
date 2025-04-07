// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::pod_config::PodConfigRepository;
use tracing::{debug, info, instrument};

use crate::{features::pod_config::PodConfigNotInitialized, AppState};

#[instrument(level = "trace", skip_all, err)]
pub async fn init_cors(
    AppState {
        db, cors_config, ..
    }: &AppState,
) -> Result<(), String> {
    debug!("Initializing CORS settings…");

    let dashboard_url = match PodConfigRepository::get_dashboard_url(db).await {
        Ok(Some(dashboard_url)) => dashboard_url,
        Ok(None) => {
            info!("Not initializing CORS settings: {PodConfigNotInitialized}");
            return Ok(());
        }
        Err(err) => {
            return Err(format!("Could not initialize CORS settings: {err}"));
        }
    };

    (cors_config.allowed_origins.write().unwrap()).insert(dashboard_url);

    tracing::debug!(
        "CORS allowed origins: {}",
        cors_config
            .allowed_origins
            .read()
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    Ok(())
}
