// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod license_validator;
pub mod licensing_controller;
pub mod licensing_service;

use biscuit::{AuthorizerBuilder, Biscuit};

pub use self::errors::*;
pub use self::license_validator::*;
pub use self::licensing_service::*;

/// See https://github.com/eclipse-biscuit/website/commit/6284753980acceabbf0e18b784d93e3bfebe53fd#r163283775.
pub trait BiscuitExt {
    fn authorize(&self, authorizer: &AuthorizerBuilder) -> Result<usize, biscuit::error::Token>;
}

impl BiscuitExt for Biscuit {
    fn authorize(&self, authorizer: &AuthorizerBuilder) -> Result<usize, biscuit::error::Token> {
        authorizer.clone().build(self)?.authorize()
    }
}

pub mod errors {
    #[derive(Debug, Clone, Copy)]
    #[derive(thiserror::Error)]
    #[error("User limit reached.")]
    pub struct UserLimitReached;

    #[derive(Debug, Clone, Copy)]
    #[derive(thiserror::Error)]
    #[error("No valid license.")]
    pub struct NoValidLicense;
}
