// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use secrecy::SecretString;
pub use service::secrets::SecretsStoreImpl;
use service::{
    models::xmpp::BareJid,
    secrets::{LiveSecretsStore, ServiceAccountSecrets},
    AppConfig,
};

#[derive(Debug)]
pub struct MockSecretsStore {
    implem: LiveSecretsStore,
    state: Arc<RwLock<MockSecretsStoreState>>,
    api_jid: BareJid,
}

#[derive(Debug, Default)]
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

    pub(crate) fn state<'a>(&'a self) -> RwLockReadGuard<'a, MockSecretsStoreState> {
        self.state
            .read()
            .expect("`MockSecretsStoreState` lock poisonned.")
    }

    pub(crate) fn state_mut<'a>(&'a self) -> RwLockWriteGuard<'a, MockSecretsStoreState> {
        self.state
            .write()
            .expect("`MockSecretsStoreState` lock poisonned.")
    }

    pub fn increase_changes_count(&self, jid: BareJid) {
        *self.state_mut().changes_count.entry(jid).or_insert(0) += 1
    }
    pub fn changes_count(&self, jid: &BareJid) -> u32 {
        self.state().changes_count.get(jid).cloned().unwrap_or(0)
    }
}

impl SecretsStoreImpl for MockSecretsStore {
    fn load_config(&self, app_config: &service::AppConfig) {
        self.implem.load_config(app_config);
    }

    fn set_prose_pod_api_xmpp_password(&self, password: SecretString) {
        self.increase_changes_count(self.api_jid.clone());
        self.implem.set_prose_pod_api_xmpp_password(password)
    }
    fn prose_pod_api_xmpp_password(&self) -> Option<SecretString> {
        self.implem.prose_pod_api_xmpp_password()
    }

    fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets) {
        self.increase_changes_count(jid.clone());
        self.implem.set_service_account_secrets(jid, secrets)
    }
    fn get_service_account_password(&self, jid: &BareJid) -> Option<SecretString> {
        self.implem.get_service_account_password(jid)
    }
    fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<SecretString> {
        self.implem.get_service_account_prosody_token(jid)
    }
    fn set_service_account_prosody_token(
        &self,
        jid: &BareJid,
        prosody_token: SecretString,
    ) -> Result<(), service::secrets::ServiceAccountNotFound> {
        self.increase_changes_count(jid.clone());
        self.implem
            .set_service_account_prosody_token(jid, prosody_token)
    }
}
