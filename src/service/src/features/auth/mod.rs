// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

//! Authentication & authorization.

pub mod auth_controller;
pub mod auth_service;
pub mod errors;
mod live_auth_service;
mod models;
mod password_reset_notification;
pub mod util;

pub use auth_service::{AuthService, AuthServiceImpl};
pub use live_auth_service::LiveAuthService;

pub use self::models::*;

pub mod password_reset_tokens {
    use anyhow::Context;
    use jid::BareJid;

    use super::{PasswordResetKvRecord, PasswordResetRecord, PasswordResetToken};

    pub use self::kv_store::{delete as kv_store_delete, get_all};

    crate::gen_scoped_kv_store!(pub(super) auth::password_reset_tokens);

    pub async fn set(
        db: &impl sea_orm::ConnectionTrait,
        key: &PasswordResetToken,
        value: &PasswordResetRecord,
    ) -> anyhow::Result<()> {
        use secrecy::ExposeSecret;

        let key = key.expose_secret();
        // NOTE: Unwrapping is safe here since we’re only serializing a
        //   UUID, a date and two Rust variable names.
        let value = serde_json::to_value(value).unwrap();

        self::kv_store::set(db, key, value).await
    }

    pub async fn get(
        db: &impl sea_orm::ConnectionTrait,
        token: &PasswordResetToken,
    ) -> anyhow::Result<Option<PasswordResetRecord>> {
        use secrecy::ExposeSecret;

        match self::kv_store::get(db, &token.expose_secret()).await? {
            Some(json) => {
                let data = serde_json::from_value::<PasswordResetRecord>(json)
                    .context("Invalid record stored")?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// Returns whether or not a record was deleted.
    pub async fn delete(
        db: &impl sea_orm::ConnectionTrait,
        token: &PasswordResetToken,
    ) -> anyhow::Result<bool> {
        use secrecy::ExposeSecret as _;
        self::kv_store::delete(db, &token.expose_secret()).await
    }

    #[tracing::instrument(skip(db), ret)]
    pub async fn get_by_jid(
        db: &impl sea_orm::ConnectionTrait,
        jid: &BareJid,
    ) -> anyhow::Result<Vec<PasswordResetToken>> {
        let mut records =
            self::kv_store::get_by_value_entry::<PasswordResetKvRecord>(db, ("jid", jid.as_str()))
                .await?;
        records.sort_by_key(|r| r.value.expires_at);
        let tokens = records.into_iter().map(|r| r.key).collect::<Vec<_>>();
        Ok(tokens)
    }
}
