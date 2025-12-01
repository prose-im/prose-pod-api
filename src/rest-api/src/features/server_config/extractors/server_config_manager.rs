// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::extractors::prelude::*;

impl FromRef<AppState> for service::server_config::ServerConfigManager {
    fn from_ref(state: &AppState) -> Self {
        Self::new(
            state.db.clone(),
            state.app_config.clone(),
            state.prose_pod_server_service.clone(),
        )
    }
}
