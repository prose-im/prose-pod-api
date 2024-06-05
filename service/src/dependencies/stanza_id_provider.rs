// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

pub trait StanzaIdProvider: Send + Sync {
    fn new_id(&self) -> String;
}

#[derive(Clone)]
pub struct UUIDStanzaIdProvider;

impl StanzaIdProvider for UUIDStanzaIdProvider {
    fn new_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

impl StanzaIdProvider for Arc<dyn StanzaIdProvider> {
    fn new_id(&self) -> String {
        self.deref().new_id()
    }
}
