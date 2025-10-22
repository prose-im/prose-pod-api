// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod app_config;
mod avatar;
mod lua;
mod notification_service;
mod prose_pod_server_service;
mod xmpp_service;

pub use lua::Lua;

pub mod prelude {
    pub use std::{convert::Infallible, sync::Arc};

    pub use axum::{
        extract::{FromRef, FromRequest, FromRequestParts, Request},
        http::request,
    };

    pub use crate::{
        error::{self, Error},
        AppState,
    };
}
