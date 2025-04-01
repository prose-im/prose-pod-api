// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// Generates a route for setting a specific Pod config.
#[macro_export]
macro_rules! pod_config_routes {
    ($var:ident, req: $req_type:ty, res: $res_type:ty $(, get: $get_route_fn:ident, get_fn: $get_repo_fn:ident)? $(, set: $set_route_fn:ident $(, validate_set: $validate:block)?)? $(,)?) => {
        $(pub async fn $set_route_fn(
            axum::extract::State(crate::AppState { db, .. }): axum::extract::State<crate::AppState>,
            axum::Json($var): axum::Json<$req_type>,
        ) -> Result<axum::Json<$res_type>, crate::error::Error> {
            use axum::Json;
            use service::pod_config::{PodConfigRepository, PodConfigUpdateForm};
            use crate::{error::Error, features::pod_config::PodConfigNotInitialized};

            $($validate)?

            if !PodConfigRepository::is_initialized(&db).await? {
                return Err(Error::from(PodConfigNotInitialized));
            }

            let model = PodConfigRepository::set(
                &db,
                PodConfigUpdateForm {
                    $var: Some($var.into()),
                    ..Default::default()
                },
            )
            .await?;

            Ok(Json(model.$var()))
        })?

        $(pub async fn $get_route_fn(
            axum::extract::State(crate::AppState { db, .. }): axum::extract::State<crate::AppState>,
        ) -> Result<axum_extra::either::Either<axum::Json<$res_type>, axum::response::NoContent>, crate::error::Error> {
            use axum::{Json, response::NoContent};
            use axum_extra::either::Either;
            use service::pod_config::PodConfigRepository;

            Ok(match PodConfigRepository::$get_repo_fn(&db).await? {
                Some($var) => Either::E1(Json(Some($var))),
                None => Either::E2(NoContent),
            })
        })?
    };
    ($var:ident, $var_type:ty $(, get: $get_route_fn:ident, get_fn: $get_repo_fn:ident)? $(, set: $set_route_fn:ident $(, validate_set: $validate:tt)?)? $(,)?) => {
        crate::pod_config_routes!(
            $var,
            req: $var_type, res: $var_type,
            $(get: $get_route_fn, get_fn: $get_repo_fn, )?
            $(set: $set_route_fn, $( validate_set: $validate, )? )?
        );
    };
}
