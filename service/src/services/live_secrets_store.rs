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
    config::AppConfig,
    services::secrets_store::{SecretsStoreImpl, ServiceAccountSecrets},
};

/// A place to store service accounts secrets (e.g. Prosody tokens).
///
/// WARN: This must NOT be used to save user tokens!
#[derive(Debug, Clone)]
pub struct LiveSecretsStore {
    store: Arc<RwLock<HashMap<BareJid, ServiceAccountSecrets>>>,
    // NOTE: This password is the only one to get a special treatment because of how
    //   the Prose Pod API interacts with the Prose Pod Server.
    prose_pod_api_xmpp_password: Arc<RwLock<SecretString>>,
}

impl LiveSecretsStore {
    pub fn from_config(app_config: &AppConfig) -> Self {
        let prose_pod_api_xmpp_password = app_config.bootstrap.prose_pod_api_xmpp_password.as_ref().expect("App config is missing `bootstrap.prose_pod_api_xmpp_password`. You should define the `PROSE_BOOTSTRAP__PROSE_POD_API_XMPP_PASSWORD` environment variable.");

        Self {
            store: Arc::default(),
            prose_pod_api_xmpp_password: Arc::new(RwLock::new(
                prose_pod_api_xmpp_password.to_owned(),
            )),
        }
    }

    fn get_secrets(&self, jid: &BareJid) -> Option<ServiceAccountSecrets> {
        self.store
            .read()
            .expect("`ServiceSecretsStore` lock poisonned.")
            .get(jid)
            .cloned()
    }
}

impl SecretsStoreImpl for LiveSecretsStore {
    fn set_prose_pod_api_xmpp_password(&self, password: SecretString) {
        *self
            .prose_pod_api_xmpp_password
            .clone()
            .write()
            .expect("`prose_pod_api_xmpp_password` lock poisonned.") = password;
    }

    fn prose_pod_api_xmpp_password(&self) -> SecretString {
        self.prose_pod_api_xmpp_password
            .read()
            .expect("`prose_pod_api_xmpp_password` lock poisonned.")
            .to_owned()
    }

    fn set_service_account_secrets(&self, jid: BareJid, secrets: ServiceAccountSecrets) {
        self.store
            .write()
            .expect("`ServiceSecretsStore` lock poisonned.")
            .insert(jid, secrets);
    }

    fn get_service_account_prosody_token(&self, jid: &BareJid) -> Option<SecretString> {
        self.get_secrets(jid).map(|c| c.prosody_token)
    }
}
