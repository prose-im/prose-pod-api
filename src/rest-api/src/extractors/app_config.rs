// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::prelude::*;

impl FromRef<AppState> for Arc<service::AppConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.app_config.clone()
    }
}
