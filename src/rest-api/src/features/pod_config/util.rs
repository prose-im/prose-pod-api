// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// Generates a route for setting a specific Pod config.
#[macro_export]
macro_rules! pod_config_route {
    (
        $route_fn:ident: get $var_name:ident
        ($res_type:ty) using $repo_fn:ident
        $(,)?
    ) => {
        pub async fn $route_fn(
            axum::extract::State(crate::AppState { db, .. }): axum::extract::State<crate::AppState>,
        ) -> Result<
            axum_extra::either::Either<axum::Json<$res_type>, axum::response::NoContent>,
            crate::error::Error,
        > {
            use axum::{response::NoContent, Json};
            use axum_extra::either::Either;
            use service::pod_config::PodConfigRepository;

            Ok(match PodConfigRepository::$repo_fn(&db).await? {
                Some($var_name) => Either::E1(Json(Some($var_name))),
                None => Either::E2(NoContent),
            })
        }
    };
    (
        $route_fn:ident: update $var_name:ident
        ($res_type:ty) with $req_type:ty
        $(, validate: $validate:block)?
        $(, then: $then:block)?
        $(,)?
    ) => {
        pub async fn $route_fn(
            axum::extract::State(crate::AppState { db, .. }): axum::extract::State<crate::AppState>,
            axum::Json($var_name): axum::Json<$req_type>,
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
                    $var_name: Some($var_name.into()),
                    ..Default::default()
                },
            )
            .await?;

            $($then)?

            Ok(Json(model.$var_name()))
        }
    };
}

/// Generates CRU(D) routes for a specific Pod config.
#[macro_export]
macro_rules! pod_config_routes {
    (
        key: $var_name:ident, type: $res_type:ty
        $(, set: $set_route_fn:ident with $set_req_type:ty
            $(, validate: $set_validate:block)?
            $(, then: $set_then:block)?
        )?
        $(, get: $get_route_fn:ident using $get_repo_fn:ident)?
        $(, patch: $patch_route_fn:ident with $patch_req_type:ty
            $(, validate: $patch_validate:block)?
            $(, then: $patch_then:block)?
        )?
        $(,)?
    ) => {
        $(crate::pod_config_route!(
            $set_route_fn: update $var_name ($res_type) with $set_req_type,
            $(validate: $set_validate,)?
            $(then: $set_then,)?
        );)?
        $(crate::pod_config_route!($get_route_fn: get $var_name ($res_type) using $get_repo_fn);)?
        $(crate::pod_config_route!(
            $patch_route_fn: update $var_name ($res_type) with $patch_req_type,
            $(validate: $patch_validate,)?
            $(then: $patch_then,)?
        );)?
    };
    (
        key: $var:ident, type: $var_type:ty
        $(, set: $set_route_fn:ident
            $(, validate: $set_validate:block)?
            $(, then: $set_then:block)?
        )?
        $(, get: $get_route_fn:ident using $get_repo_fn:ident)?
        $(,)?
    ) => {
        crate::pod_config_routes!(
            key: $var, type: $var_type,
            $(
                set: $set_route_fn with $var_type,
                $(validate: $set_validate,)?
                $(then: $set_then,)?
            )?
            $(get: $get_route_fn using $get_repo_fn,)?
        );
    };
}
