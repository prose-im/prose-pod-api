// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod lua;
mod server_config;

pub use lua::Lua;

pub mod prelude {
    pub use axum::{extract::FromRequestParts, http::request};

    pub use crate::{
        error::{self, Error},
        AppState,
    };
}
