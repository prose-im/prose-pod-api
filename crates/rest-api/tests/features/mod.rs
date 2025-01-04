// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod auth;
pub mod dns_setup;
pub mod init;
pub mod invitations;
pub mod members;
pub mod network_checks;
pub mod pod_config;
pub mod roles;
pub mod server_config;
pub mod workspace_details;

pub(crate) mod prelude {
    pub(crate) use std::{str::FromStr as _, sync::Arc};

    pub(crate) use axum::http::{header::*, StatusCode};
    pub(crate) use axum_test::{TestResponse, TestServer};
    pub(crate) use base64::{engine::general_purpose::URL_SAFE_NO_PAD as Base64, Engine as _};
    pub(crate) use chrono::{TimeDelta, Utc};
    pub(crate) use cucumber::*;
    pub(crate) use prose_pod_api::error::Error;
    pub(crate) use secrecy::{ExposeSecret as _, SecretString};
    pub(crate) use serde_json::json;
    pub(crate) use service::{
        errors::DbErr,
        models::xmpp::*,
        sea_orm::{prelude::*, IntoActiveModel as _, Set},
        server_config::ServerConfigCreateForm,
        MutationError,
    };

    pub(crate) use crate::{
        api_call_fn, cucumber_parameters as parameters, user_token, util::*, TestWorld,
    };
}
