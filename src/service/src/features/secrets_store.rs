// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod prelude {
    pub use std::sync::Arc;

    pub use jid::BareJid;

    pub use crate::{
        auth::{AuthToken, Password},
        AppConfig,
    };

    pub use super::{SecretsStoreImpl, ServiceAccountNotFound, ServiceAccountSecrets};
}

pub use self::live_secrets_store::LiveSecretsStore;
use self::prelude::*;

/// A place to store service accounts secrets (e.g. Prosody tokens).
///
/// WARN: This must NOT be used to save user tokens!
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct SecretsStore(pub Arc<dyn SecretsStoreImpl>);

#[derive(Debug, Clone)]
pub struct ServiceAccountSecrets {
    /// NOTE: We have to store the password so tokens can be refreshed.
    ///   It’s unfortunate but we could just rotate them very often if we want to.
    pub password: Password,
    pub prosody_token: AuthToken,
}

pub trait SecretsStoreImpl: std::fmt::Debug + Sync + Send {
    /// Load app configuration. This is useful to load the bootstrapping
    /// password during the first startup. On subsequent startups, nothing
    /// should change (to allow the API to restart properly).
    fn load_config(&self, app_config: &AppConfig);

    fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets);
    fn get_service_account_password(&self, jid: &BareJid) -> Option<Password>;
    fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<AuthToken>;
    fn set_service_account_prosody_token(
        &self,
        jid: &BareJid,
        prosody_token: AuthToken,
    ) -> Result<(), ServiceAccountNotFound>;
}

#[derive(Debug, thiserror::Error)]
#[error("Service account not found.")]
pub struct ServiceAccountNotFound;

mod live_secrets_store {
    use std::{collections::HashMap, sync::RwLock};

    use super::prelude::*;

    /// A place to store service accounts secrets (e.g. Prosody tokens).
    ///
    /// WARN: This must NOT be used to save user tokens!
    #[derive(Debug, Default)]
    pub struct LiveSecretsStore {
        store: Arc<RwLock<HashMap<BareJid, ServiceAccountSecrets>>>,
    }

    impl LiveSecretsStore {
        fn get_secrets(&self, jid: &BareJid) -> Option<ServiceAccountSecrets> {
            (self.store.read())
                .expect("`ServiceSecretsStore` lock poisonned.")
                .get(jid)
                .cloned()
        }
    }

    impl SecretsStoreImpl for LiveSecretsStore {
        fn load_config(&self, _app_config: &AppConfig) {
            // NOTE: While this is unused at the moment, it will likely be
            //   useful again soon therefore we’ll keep the logic in place.
        }

        fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets) {
            (self.store.write())
                .expect("`ServiceSecretsStore` lock poisonned.")
                .insert(jid, secrets);
        }

        fn get_service_account_password(&self, jid: &BareJid) -> Option<Password> {
            self.get_secrets(jid).map(|c| c.password)
        }
        fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<AuthToken> {
            self.get_secrets(jid).map(|c| c.prosody_token)
        }
        fn set_service_account_prosody_token(
            &self,
            jid: &BareJid,
            prosody_token: AuthToken,
        ) -> Result<(), ServiceAccountNotFound> {
            (self.store.write())
                .expect("`ServiceSecretsStore` lock poisonned.")
                .get_mut(jid)
                .ok_or(ServiceAccountNotFound)?
                .prosody_token = prosody_token;
            Ok(())
        }
    }
}

// MARK: - Boilerplate

impl std::ops::Deref for SecretsStore {
    type Target = Arc<dyn SecretsStoreImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
