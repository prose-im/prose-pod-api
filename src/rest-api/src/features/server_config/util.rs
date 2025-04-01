// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_export]
macro_rules! server_config_route {
    (set: $var_type:ty, $var:ident, $route_fn:ident, $manager_fn:ident) => {
        pub async fn $route_fn(
            server_manager: service::xmpp::ServerManager,
            axum::Json($var): axum::Json<$var_type>,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            server_manager.$manager_fn($var.clone()).await?;
            Ok(axum::Json($var))
        }
    };
    (get: $var_type:ty, $var:ident, $route_fn:ident) => {
        pub async fn $route_fn(
            server_config: service::server_config::ServerConfig,
        ) -> axum::Json<$var_type> {
            axum::Json(server_config.$var)
        }
    };
    (reset: $var_type:ty, $var:ident, $route_fn:ident, $manager_fn:ident) => {
        pub async fn $route_fn(
            server_manager: service::xmpp::ServerManager,
        ) -> Result<axum::Json<$var_type>, crate::error::Error> {
            let $var = server_manager.$manager_fn().await?.$var;
            Ok(axum::Json($var))
        }
    };
}

/// Generates routes for setting, querying and resetting a specific server config.
#[macro_export]
macro_rules! server_config_routes {
    (
        key: $var_name:ident, type: $res_type:ty
        $(,   set:   $set_route_fn:ident using   $set_manager_fn:ident)?
        $(,   get:   $get_route_fn:ident                              )?
        $(, reset: $reset_route_fn:ident using $reset_manager_fn:ident)?
        $(,)?
    ) => {
        $(crate::server_config_route!(  set: $res_type, $var_name,   $set_route_fn,   $set_manager_fn);)?
        $(crate::server_config_route!(  get: $res_type, $var_name,   $get_route_fn                   );)?
        $(crate::server_config_route!(reset: $res_type, $var_name, $reset_route_fn, $reset_manager_fn);)?
    };
}

/// Generates a route for resetting a group of server configs.
#[macro_export]
macro_rules! server_config_reset_route {
    ($fn:ident, $route_fn:ident $(,)?) => {
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
