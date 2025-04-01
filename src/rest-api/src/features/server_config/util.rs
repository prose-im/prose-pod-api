// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// Generates a route for setting a specific server config.
/// Also generates its associated request type.
#[macro_export]
macro_rules! server_config_set_route {
    ($var_type:ty, $var:ident, $fn:ident, $route_fn:ident) => {
        pub async fn $route_fn(
            server_manager: service::xmpp::ServerManager,
            axum::Json(new_state): axum::Json<$var_type>,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            server_manager.$fn(new_state.clone()).await?;
            Ok(axum::Json(new_state))
        }
    };
}

/// Generates a route for resetting a specific server config.
#[macro_export]
macro_rules! server_config_reset_route {
    ($fn:ident, $route_fn:ident) => {
        pub async fn $route_fn(
            server_manager: service::xmpp::ServerManager,
        ) -> Result<
            (
                [(axum::http::HeaderName, axum::http::HeaderValue); 1],
                axum::Json<service::server_config::ServerConfig>,
            ),
            crate::error::Error,
        > {
            let new_config = server_manager.$fn().await?;
            Ok((
                [(
                    axum::http::header::CONTENT_LOCATION,
                    axum::http::HeaderValue::from_static(
                        crate::features::init::SERVER_CONFIG_ROUTE,
                    ),
                )],
                axum::Json(new_config),
            ))
        }
    };
}
