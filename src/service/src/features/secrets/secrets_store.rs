// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, ops::Deref, sync::Arc};

use jid::BareJid;
use secrecy::SecretString;

/// A place to store service accounts secrets (e.g. Prosody tokens).
///
/// WARN: This must NOT be used to save user tokens!
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct SecretsStore(Arc<dyn SecretsStoreImpl>);

impl SecretsStore {
    pub fn new(implem: Arc<dyn SecretsStoreImpl>) -> Self {
        Self(implem)
    }
}

impl Deref for SecretsStore {
    type Target = Arc<dyn SecretsStoreImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ServiceAccountSecrets {
    /// NOTE: We have to store the password so tokens can be refreshed.
    ///   It’s unfortunate but we could just rotate them very often if we want to.
    pub password: SecretString,
    pub prosody_token: SecretString,
}

pub trait SecretsStoreImpl: Debug + Sync + Send {
    /// Load app configuration. This is useful to load the bootstrapping
    /// password during the first startup. On subsequent startups, nothing
    /// should change (to allow the API to restart properly).
    fn load_config(&self, app_config: &crate::AppConfig);

    fn set_prose_pod_api_xmpp_password(&self, password: SecretString);
    fn prose_pod_api_xmpp_password(&self) -> Option<SecretString>;

    fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets);
    fn get_service_account_password(&self, jid: &BareJid) -> Option<SecretString>;
    fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<SecretString>;
    fn set_service_account_prosody_token(
        &self,
        jid: &BareJid,
        prosody_token: SecretString,
    ) -> Result<(), ServiceAccountNotFound>;
}

#[derive(Debug, thiserror::Error)]
#[error("Service account not found.")]
pub struct ServiceAccountNotFound;
