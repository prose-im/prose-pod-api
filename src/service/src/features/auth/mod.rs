// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Authentication & authorization.

pub mod auth_controller;
pub mod auth_service;
pub mod errors;
pub mod live_auth_service;
mod models;
mod password_reset_notification;
pub mod util;

pub use auth_service::{AuthService, AuthServiceImpl};
pub use live_auth_service::LiveAuthService;

pub use self::models::*;

pub mod password_reset_tokens {
    use anyhow::Context;
    use sea_orm::ConnectionTrait;

    use crate::auth::PasswordResetRecord;

    pub use self::kv_store::KvStore;

    use super::PasswordResetToken;

    crate::gen_scoped_kv_store!("auth::password_reset_tokens");

    impl KvStore {
        pub(crate) async fn set_token(
            db: &impl ConnectionTrait,
            key: &PasswordResetToken,
            value: PasswordResetRecord,
        ) -> anyhow::Result<()> {
            use secrecy::ExposeSecret;

            let key = key.expose_secret();
            // NOTE: Unwrapping is safe here since we’re only serializing a
            //   UUID, a date and two Rust variable names.
            let value = serde_json::to_value(&value).unwrap();

            Self::set(db, key, value).await
        }

        pub(crate) async fn get_token_data(
            db: &impl ConnectionTrait,
            token: &PasswordResetToken,
        ) -> anyhow::Result<Option<PasswordResetRecord>> {
            use secrecy::ExposeSecret;

            match Self::get(db, &token.expose_secret()).await? {
                Some(json) => {
                    let data = serde_json::from_value::<PasswordResetRecord>(json)
                        .context("Invalid record stored")?;
                    Ok(Some(data))
                }
                None => Ok(None),
            }
        }
    }
}
