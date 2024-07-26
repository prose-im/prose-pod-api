// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;

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
        $world
            .members
            .get(&$name)
            .expect("User must be created first")
            .1
            .clone()
    };
}

#[macro_export]
macro_rules! basic_auth_api_call_fn {
    ($fn:ident, $method:ident, $route:expr) => {
        async fn $fn<'a>(
            client: &'a rocket::local::asynchronous::Client,
            username: impl Display,
            token: impl secrecy::ExposeSecret<String>,
        ) -> rocket::local::asynchronous::LocalResponse<'a> {
            client
                .$method($route)
                .header(rocket::http::ContentType::JSON)
                .header(rocket::http::Header::new(
                    "Authorization",
                    format!("Basic {}", token.expose_secret()),
                ))
                .dispatch()
                .await
        }
    };
}
#[macro_export]
macro_rules! api_call_fn {
    ($fn:ident, $method:ident, $route:expr) => {
        async fn $fn<'a>(
            client: &'a rocket::local::asynchronous::Client,
            token: impl secrecy::ExposeSecret<String>,
        ) -> rocket::local::asynchronous::LocalResponse<'a> {
            client
                .$method($route)
                .header(rocket::http::ContentType::JSON)
                .header(rocket::http::Header::new(
                    "Authorization",
                    format!("Bearer {}", token.expose_secret()),
                ))
                .dispatch()
                .await
        }
    };
}
#[macro_export]
macro_rules! api_call_with_body_fn {
    ($fn:ident, $method:ident, $route:expr, $payload_type:ident, $var:ident, $var_type:ty) => {
        async fn $fn<'a>(
            client: &'a rocket::local::asynchronous::Client,
            token: impl secrecy::ExposeSecret<String>,
            state: $var_type,
        ) -> rocket::local::asynchronous::LocalResponse<'a> {
            client
                .$method($route)
                .header(rocket::http::ContentType::JSON)
                .header(rocket::http::Header::new(
                    "Authorization",
                    format!("Bearer {}", token.expose_secret()),
                ))
                .body(serde_json::json!($payload_type { $var: state.into() }).to_string())
                .dispatch()
                .await
        }
    };
}
