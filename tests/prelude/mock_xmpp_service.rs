// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::JID;
use ::service::xmpp_service::{Error, XmppServiceImpl};
use linked_hash_map::LinkedHashMap;
use service::{prose_xmpp::mods::AvatarData, xmpp_service::VCard, XmppServiceContext};

use std::{fmt::Debug, sync::RwLock};

#[derive(Debug)]
pub struct MockXmppService {
    pub(crate) online: bool,
    pub(crate) state: RwLock<MockXmppServiceState>,
}

#[derive(Debug, Default)]
pub struct MockXmppServiceState {
    pub vcards: LinkedHashMap<JID, VCard>,
    pub avatars: LinkedHashMap<JID, Option<AvatarData>>,
}

impl MockXmppService {
    pub fn new(state: RwLock<MockXmppServiceState>) -> Self {
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
        Self::new(RwLock::default())
    }
}

impl MockXmppService {
    pub fn get_vcard(&self, jid: &JID) -> Result<Option<VCard>, Error> {
        self.check_online()?;

        Ok(self
            .state
            .read()
            .unwrap()
            .vcards
            .get(jid)
            .map(ToOwned::to_owned))
    }
    pub fn set_vcard(&self, jid: &JID, vcard: &VCard) -> Result<(), Error> {
        self.check_online()?;

        self.state
            .write()
            .unwrap()
            .vcards
            .insert(jid.to_owned(), vcard.to_owned());
        Ok(())
    }

    pub fn get_avatar(&self, jid: &JID) -> Result<Option<AvatarData>, Error> {
        self.check_online()?;

        Ok(self
            .state
            .read()
            .unwrap()
            .avatars
            .get(jid)
            .cloned()
            .flatten())
    }
    pub fn set_avatar(&self, jid: &JID, image_data: Option<Vec<u8>>) -> Result<(), Error> {
        self.check_online()?;

        self.state.write().unwrap().avatars.insert(
            jid.to_owned(),
            image_data.map(|d| AvatarData::Base64(String::from_utf8(d).unwrap())),
        );
        Ok(())
    }
}

impl XmppServiceImpl for MockXmppService {
    fn get_vcard(&self, _ctx: &XmppServiceContext, jid: &JID) -> Result<Option<VCard>, Error> {
        self.get_vcard(jid)
    }
    fn set_own_vcard(&self, ctx: &XmppServiceContext, vcard: &VCard) -> Result<(), Error> {
        self.set_vcard(&ctx.full_jid, vcard)
    }

    fn get_avatar(
        &self,
        _ctx: &XmppServiceContext,
        jid: &JID,
    ) -> Result<Option<AvatarData>, Error> {
        self.get_avatar(jid)
    }
    fn set_own_avatar(&self, ctx: &XmppServiceContext, image_data: Vec<u8>) -> Result<(), Error> {
        self.set_avatar(&ctx.full_jid, Some(image_data))
    }
}
