// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::entity::model::JID;
use ::service::xmpp_service::{Error, XmppServiceImpl};
use linked_hash_map::LinkedHashMap;
use service::{
    xmpp::stanza::avatar::{self, AvatarData},
    xmpp_parsers::hashes::Sha1HexAttribute,
    VCard, XmppServiceContext,
};

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
    pub avatars: LinkedHashMap<JID, AvatarData>,
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

impl MockXmppService {
    pub fn get_vcard(&self, jid: &JID) -> Result<Option<VCard>, Error> {
        self.check_online()?;

        Ok(self
            .state
            .lock()
            .unwrap()
            .vcards
            .get(jid)
            .map(ToOwned::to_owned))
    }
    pub fn set_vcard(&self, jid: &JID, vcard: &VCard) -> Result<(), Error> {
        self.check_online()?;

        self.state
            .lock()
            .unwrap()
            .vcards
            .insert(jid.to_owned(), vcard.to_owned());
        Ok(())
    }

    pub fn get_avatar(&self, jid: &JID) -> Result<Option<AvatarData>, Error> {
        self.check_online()?;

        Ok(self
            .state
            .lock()
            .unwrap()
            .avatars
            .get(jid)
            .map(ToOwned::to_owned))
    }
    pub fn set_avatar(&self, jid: &JID, base64_image_data: String) -> Result<(), Error> {
        self.check_online()?;

        self.state
            .lock()
            .unwrap()
            .avatars
            .insert(jid.to_owned(), AvatarData::Base64(base64_image_data));
        Ok(())
    }
}

impl XmppServiceImpl for MockXmppService {
    fn get_vcard(&self, _ctx: &XmppServiceContext, jid: &JID) -> Result<Option<VCard>, Error> {
        self.check_online()?;
        self.get_vcard(jid)
    }
    fn set_vcard(&self, _ctx: &XmppServiceContext, jid: &JID, vcard: &VCard) -> Result<(), Error> {
        self.check_online()?;
        self.set_vcard(jid, vcard)
    }

    fn get_avatar(
        &self,
        _ctx: &XmppServiceContext,
        jid: &JID,
        _image_id: &Sha1HexAttribute,
    ) -> Result<Option<AvatarData>, Error> {
        self.get_avatar(jid)
    }
    fn set_avatar(
        &self,
        _ctx: &XmppServiceContext,
        jid: &JID,
        _checksum: &avatar::ImageId,
        base64_image_data: String,
    ) -> Result<(), Error> {
        self.set_avatar(jid, base64_image_data)
    }
}
