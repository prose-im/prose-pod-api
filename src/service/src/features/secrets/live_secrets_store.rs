// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use jid::BareJid;
use secrecy::SecretString;

use crate::{
    secrets::{SecretsStoreImpl, ServiceAccountSecrets},
    AppConfig,
};

use super::ServiceAccountNotFound;

/// A place to store service accounts secrets (e.g. Prosody tokens).
///
/// WARN: This must NOT be used to save user tokens!
#[derive(Debug, Default)]
pub struct LiveSecretsStore {
    store: Arc<RwLock<HashMap<BareJid, ServiceAccountSecrets>>>,
    // NOTE: This account is the only one to get a special treatment because
    //   of how the Prose Pod API interacts with the Prose Pod Server.
    prose_pod_api_xmpp_password: Arc<RwLock<Option<SecretString>>>,
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
    fn load_config(&self, app_config: &AppConfig) {
        if self.prose_pod_api_xmpp_password().is_none() {
            let password = app_config.bootstrap.prose_pod_api_xmpp_password.clone();
            self.set_prose_pod_api_xmpp_password(password);
        }
    }

    fn set_prose_pod_api_xmpp_password(&self, password: SecretString) {
        *(self.prose_pod_api_xmpp_password.write())
            .expect("`prose_pod_api_xmpp_password` lock poisonned.") = Some(password);
    }

    fn prose_pod_api_xmpp_password(&self) -> Option<SecretString> {
        (self.prose_pod_api_xmpp_password.read())
            .expect("`prose_pod_api_xmpp_password` lock poisonned.")
            .to_owned()
    }

    fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets) {
        (self.store.write())
            .expect("`ServiceSecretsStore` lock poisonned.")
            .insert(jid, secrets);
    }

    fn get_service_account_password(&self, jid: &BareJid) -> Option<SecretString> {
        self.get_secrets(jid).map(|c| c.password)
    }
    fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<SecretString> {
        self.get_secrets(jid).map(|c| c.prosody_token)
    }
    fn set_service_account_prosody_token(
        &self,
        jid: &BareJid,
        prosody_token: SecretString,
    ) -> Result<(), ServiceAccountNotFound> {
        (self.store.write())
            .expect("`ServiceSecretsStore` lock poisonned.")
            .get_mut(jid)
            .ok_or(ServiceAccountNotFound)?
            .prosody_token = prosody_token;
        Ok(())
    }
}
