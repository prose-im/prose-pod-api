// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, str::FromStr as _};

use prose_pod_api::error::{self, Error};
use service::xmpp::BareJid;

use crate::TestWorld;

pub async fn name_to_jid(world: &TestWorld, name: &str) -> Result<BareJid, Error> {
    // Strip potential `<>` around the JID (if `name` is a JID).
    let name = name
        .strip_prefix("<")
        .and_then(|name| name.strip_suffix(">"))
        .unwrap_or(name);
    // Use JID if it already is one.
    if name.contains("@") {
        if let Ok(jid) = BareJid::from_str(name) {
            return Ok(jid);
        }
    }

    let domain = world.server_config().await?.domain;
    Ok(BareJid::new(&format!("{name}@{domain}")).map_err(|err| {
        error::InternalServerError(format!(
            "'{name}' cannot be used in a JID (or '{domain}' isn't a valid domain): {err}"
        ))
    })?)
}

pub fn assert_contains_if<S: Debug, T: Debug + ?Sized>(
    condition: bool,
    set: &S,
    value: &T,
    contains: impl Fn(&S, &T) -> bool,
) {
    if condition {
        assert!(contains(set, value), "{value:?} not found in {set:#?}");
    } else {
        assert!(!contains(set, value), "{value:?} found in {set:#?}");
    }
}
pub fn assert_defined_if<T: PartialEq + Debug>(condition: bool, value: Option<T>) {
    if condition {
        assert_ne!(value, None);
    } else {
        assert_eq!(value, None);
    }
}

#[macro_export]
macro_rules! user_token {
    ($world:expr, $name:expr) => {
        ($world.members.get(&$name))
            .expect("User must be created first")
            .1
            .clone()
    };
}

#[macro_export]
macro_rules! api_call_fn {
    // Authenticated.
    (
        $fn:ident,
        $method:ident,
        $route:literal $(; $route_param_name:tt=$route_param:ty)*
        $(, accept: $accept:literal)?
        $(, payload: $payload_type:ty)?
        $(,)?
    ) => {
        pub async fn $fn(
            api: &axum_test::TestServer,
            token: secrecy::SecretString,
            $($route_param_name: $route_param,)*
            $(payload: $payload_type,)?
        ) -> Result<axum_test::TestResponse, tokio::time::error::Elapsed> {
            use secrecy::ExposeSecret as _;
            tokio::time::timeout(
                tokio::time::Duration::from_millis(250),
                api.method(
                        axum::http::Method::$method,
                        &format!(
                            $route
                            $(, $route_param_name=urlencoding::encode(&$route_param_name.to_string()))*
                        )
                    )
                    .add_header(
                        axum::http::header::AUTHORIZATION,
                        format!("Bearer {}", token.expose_secret()),
                    )
                    $(.add_header(axum::http::header::ACCEPT, $accept))?
                    $(.json(&serde_json::json!(payload as $payload_type)))?,
            )
            .await
        }
    };
    // Unauthenticated.
    (
        $fn:ident,
        unauthenticated: $method:ident,
        $route:literal $(; $route_param_name:tt=$route_param:ty)*
        $(, accept: $accept:literal)?
        $(, payload: $payload_type:ty)?
        $(,)?
    ) => {
        pub async fn $fn(
            api: &axum_test::TestServer,
            $($route_param_name: $route_param,)*
            $(payload: $payload_type,)?
        ) -> Result<axum_test::TestResponse, tokio::time::error::Elapsed> {
            tokio::time::timeout(
                tokio::time::Duration::from_millis(250),
                api.method(
                        axum::http::Method::$method,
                        &format!(
                            $route
                            $(, $route_param_name=urlencoding::encode(&$route_param_name.to_string()))*
                        )
                    )
                    $(.add_header(axum::http::header::ACCEPT, $accept))?
                    $(.json(&serde_json::json!(payload as $payload_type)))?,
            )
            .await
        }
    };
    // Non-JSON payload.
    (
        $fn:ident,
        $method:ident,
        $route:literal $(; $route_param_name:tt=$route_param:ty)*,
        raw: $payload_type:ty
        $(, content_type: $content_type:literal)?
        $(,)?
    ) => {
        pub async fn $fn(
            api: &axum_test::TestServer,
            token: secrecy::SecretString,
            $($route_param_name: $route_param,)*
            payload: $payload_type,
        ) -> Result<axum_test::TestResponse, tokio::time::error::Elapsed> {
            use secrecy::ExposeSecret as _;
            tokio::time::timeout(
                tokio::time::Duration::from_millis(250),
                api.method(
                        axum::http::Method::$method,
                        &format!(
                            $route
                            $(, $route_param_name=urlencoding::encode(&$route_param_name.to_string()))*
                        )
                    )
                    .add_header(
                        axum::http::header::AUTHORIZATION,
                        format!("Bearer {}", token.expose_secret()),
                    )
                    .text(payload)
                    $(.content_type($content_type))?,
            )
            .await
        }
    };
}
