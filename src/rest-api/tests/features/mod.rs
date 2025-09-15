// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod app_config;
pub mod auth;
pub mod dns_setup;
pub mod emails;
pub mod init;
pub mod invitations;
pub mod members;
pub mod network_checks;
pub mod onboarding;
pub mod profiles;
pub mod roles;
pub mod server_config;
#[cfg(feature = "test")]
pub mod user_limit;
pub mod util;
pub mod workspace_details;

mod prelude {
    pub(super) use std::str::FromStr as _;

    pub(super) use axum::http::{header::*, StatusCode};
    pub(super) use axum_test::{TestResponse, TestServer};
    pub(super) use base64::{prelude::BASE64_STANDARD, Engine as _};
    pub(super) use chrono::{TimeDelta, Utc};
    pub(super) use cucumber::{given, then, when};
    pub(super) use prose_pod_api::error::Error;
    pub(super) use secrecy::{ExposeSecret as _, SecretString};
    pub(super) use serde_json::json;
    pub(super) use service::{
        errors::DbErr,
        models::xmpp::*,
        sea_orm::{prelude::*, IntoActiveModel as _, Set},
        MutationError,
    };

    pub(super) use crate::{api_call_fn, user_token, util::*};
    pub(super) use crate::{cucumber_parameters as parameters, TestWorld};
}
