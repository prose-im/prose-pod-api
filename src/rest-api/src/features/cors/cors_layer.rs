// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::http::HeaderValue;
use tower::Layer;
use tower_http::cors::{AllowOrigin, CorsLayer as TowerCorsLayer};

use super::CorsConfig;

#[derive(Debug, Clone)]
pub struct CorsLayer {
    pub cors_config: CorsConfig,
}

impl From<&CorsConfig> for TowerCorsLayer {
    fn from(CorsConfig { allowed_origins }: &CorsConfig) -> Self {
        let allowed_origins = allowed_origins.read().unwrap();
        let allow_origin = if allowed_origins.is_empty() {
            // NOTE: By default, until the Dashboard URL is set,
            //   the API should allow all origins.
            tracing::debug!("Access-Control-Allow-Origin: *");
            AllowOrigin::any()
        } else {
            tracing::debug!(
                "Access-Control-Allow-Origin: {}",
                allowed_origins
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            );
            AllowOrigin::list(
                allowed_origins
                    .iter()
                    .map(|url| HeaderValue::from_str(url.as_str()).unwrap()),
            )
        };
        TowerCorsLayer::new().allow_origin(allow_origin)
    }
}

impl<S> Layer<S> for CorsLayer {
    type Service = <TowerCorsLayer as Layer<S>>::Service;

    fn layer(&self, inner: S) -> Self::Service {
        tracing::debug!("CorsLayer::layer");
        TowerCorsLayer::from(&self.cors_config).layer(inner)
    }
}
