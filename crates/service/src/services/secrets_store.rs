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
    pub prosody_token: SecretString,
}

pub trait SecretsStoreImpl: Debug + Sync + Send {
    fn set_prose_pod_api_xmpp_password(&self, password: SecretString);
    fn prose_pod_api_xmpp_password(&self) -> SecretString;

    fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets);
    fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<SecretString>;
}
