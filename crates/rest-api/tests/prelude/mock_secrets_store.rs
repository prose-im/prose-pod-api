// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use secrecy::SecretString;
use service::{
    features::secrets::{LiveSecretsStore, SecretsStoreImpl, ServiceAccountSecrets},
    models::xmpp::BareJid,
    AppConfig,
};

#[derive(Debug, Clone)]
pub struct MockSecretsStore {
    implem: LiveSecretsStore,
    state: Arc<RwLock<MockSecretsStoreState>>,
    api_jid: BareJid,
}

#[derive(Debug, Clone, Default)]
pub struct MockSecretsStoreState {
    pub changes_count: HashMap<BareJid, u32>,
}

impl MockSecretsStore {
    pub fn new(implem: LiveSecretsStore, app_config: &AppConfig) -> Self {
        Self {
            implem,
            state: Default::default(),
            api_jid: app_config.api_jid(),
        }
    }

    pub(crate) fn state(&self) -> MockSecretsStoreState {
        self.state
            .read()
            .expect("`MockSecretsStoreState` lock poisonned.")
            .clone()
    }

    pub(crate) fn state_mut(&self) -> RwLockWriteGuard<MockSecretsStoreState> {
        self.state
            .write()
            .expect("`MockSecretsStoreState` lock poisonned.")
    }

    pub fn changes_count(&self, jid: &BareJid) -> u32 {
        self.state().changes_count.get(jid).cloned().unwrap_or(0)
    }
}

impl SecretsStoreImpl for MockSecretsStore {
    fn set_prose_pod_api_xmpp_password(&self, password: SecretString) {
        *self
            .state_mut()
            .changes_count
            .entry(self.api_jid.clone())
            .or_insert(0) += 1;
        self.implem.set_prose_pod_api_xmpp_password(password)
    }
    fn prose_pod_api_xmpp_password(&self) -> SecretString {
        self.implem.prose_pod_api_xmpp_password()
    }

    fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets) {
        *self
            .state_mut()
            .changes_count
            .entry(jid.clone())
            .or_insert(0) += 1;
        self.implem.set_service_account_secrets(jid, secrets)
    }
    fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<SecretString> {
        self.implem.get_service_account_prosody_token(jid)
    }
}
