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

    pub use self::kv_store::KvStore;

    use super::{PasswordResetKvRecord, PasswordResetRecord, PasswordResetToken};

    crate::gen_scoped_kv_store!("auth::password_reset_tokens");

    impl KvStore {
        pub(crate) async fn set_token(
            db: &impl ConnectionTrait,
            record: PasswordResetRecord,
        ) -> anyhow::Result<()> {
            let record = PasswordResetKvRecord::from(record);

            let key = record.key.to_string();
            // NOTE: Unwrapping is safe here since we’re only serializing a
            //   UUID, a date and two Rust variable names.
            let value = serde_json::to_value(&record.value).unwrap();

            Self::set(db, &key, value).await
        }

        pub(crate) async fn get_record(
            db: &impl ConnectionTrait,
            token: &PasswordResetToken,
        ) -> anyhow::Result<Option<PasswordResetRecord>> {
            use secrecy::ExposeSecret;

            let entry = ("token", token.expose_secret());
            match Self::get_by_value_entry(db, entry).await? {
                Some(model) => {
                    let record = PasswordResetKvRecord::try_from(model.to_owned())
                        .context("Invalid record stored")?;
                    Ok(Some(record.into()))
                }
                None => Ok(None),
            }
        }
    }
}
