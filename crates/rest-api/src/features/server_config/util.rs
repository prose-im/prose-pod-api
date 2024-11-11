// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// Generates a route for setting a specific server config.
/// Also generates its associated request type.
#[macro_export]
macro_rules! server_config_set_route {
    ($route:expr, $req_type:ident, $var_type:ty, $var:ident, $fn:ident, $route_fn:ident) => {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct $req_type {
            pub $var: $var_type,
        }

        #[rocket::put($route, format = "json", data = "<req>")]
        pub async fn $route_fn(
            server_manager: LazyGuard<ServerManager<'_>>,
            req: Json<$req_type>,
        ) -> Result<Json<ServerConfig>, crate::error::Error> {
            let server_manager = server_manager.inner?;
            let new_state: $var_type = req.$var.to_owned();
            let new_config = server_manager.$fn(new_state).await?;
            Ok(new_config.into())
        }
    };
}

/// Generates a route for resetting a specific server config.
#[macro_export]
macro_rules! server_config_reset_route {
    ($route:expr, $fn:ident, $route_fn:ident) => {
        #[rocket::put($route)]
        pub async fn $route_fn(
            server_manager: LazyGuard<ServerManager<'_>>,
        ) -> Result<Json<ServerConfig>, crate::error::Error> {
            let server_manager = server_manager.inner?;
            let new_config = server_manager.$fn().await?;
            Ok(new_config.into())
        }
    };
}
