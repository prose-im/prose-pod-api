// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, RwLock};

use service::{models::Url, AppConfig, LinkedHashSet};

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Arc<RwLock<LinkedHashSet<Url>>>,
}

impl CorsConfig {
    pub fn from_config(app_config: &AppConfig) -> Self {
        Self {
            allowed_origins: Arc::new(RwLock::new(app_config.cors.allowed_origins.clone())),
        }
    }
}
