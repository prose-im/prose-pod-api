// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, sync::RwLock};

use jid::BareJid;
use secrecy::SecretString;

/// A place to store service accounts secrets (e.g. Prosody tokens).
///
/// WARN: This must NOT be used to save user tokens!
#[derive(Default)]
pub struct ServiceSecretsStore {
    store: RwLock<HashMap<BareJid, ServiceSecrets>>,
}

#[derive(Clone)]
pub struct ServiceSecrets {
    pub prosody_token: SecretString,
}

impl ServiceSecretsStore {
    fn get_secrets(&self, jid: &BareJid) -> Option<ServiceSecrets> {
        self.store
            .read()
            .expect("`CredentialsStore` lock poisonned.")
            .get(jid)
            .cloned()
    }

    pub fn set_secrets(&self, jid: BareJid, secrets: ServiceSecrets) {
        self.store
            .write()
            .expect("`CredentialsStore` lock poisonned.")
            .insert(jid, secrets);
    }

    pub fn get_prosody_token(&self, jid: &BareJid) -> Option<SecretString> {
        self.get_secrets(jid).map(|c| c.prosody_token)
    }
}
