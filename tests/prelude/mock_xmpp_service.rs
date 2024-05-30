// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::JID;
use ::service::xmpp_service::{Error, XmppServiceImpl};
use linked_hash_map::LinkedHashMap;
use service::{VCard, XmppServiceContext};

use std::sync::Mutex;

#[derive(Debug)]
pub struct MockXmppService {
    pub(crate) online: bool,
    pub(crate) state: Mutex<MockXmppServiceState>,
}

#[derive(Debug)]
pub struct UserAccount {
    pub jid: JID,
    pub password: String,
}

#[derive(Debug, Default)]
pub struct MockXmppServiceState {
    pub vcards: LinkedHashMap<JID, VCard>,
}

impl MockXmppService {
    pub fn new(state: Mutex<MockXmppServiceState>) -> Self {
        Self {
            online: true,
            state,
        }
    }

    fn check_online(&self) -> Result<(), Error> {
        if self.online {
            Ok(())
        } else {
            Err(Error::Other("XMPP server offline".to_owned()))?
        }
    }
}

impl Default for MockXmppService {
    fn default() -> Self {
        Self::new(Mutex::default())
    }
}

impl XmppServiceImpl for MockXmppService {
    fn get_vcard(&self, _ctx: &XmppServiceContext, jid: &JID) -> Result<Option<VCard>, Error> {
        self.check_online()?;

        Ok(self.state.lock().unwrap().vcards.get(jid).map(Clone::clone))
    }
    fn set_vcard(&self, _ctx: &XmppServiceContext, jid: &JID, vcard: &VCard) -> Result<(), Error> {
        self.check_online()?;

        self.state
            .lock()
            .unwrap()
            .vcards
            .insert(jid.clone(), vcard.clone());
        Ok(())
    }
}
