// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_map::LinkedHashMap;
use service::xmpp::{
    xmpp_service::Error, AvatarData, BareJid, VCard, XmppServiceContext, XmppServiceImpl,
};
use tracing::trace;

use std::{
    collections::HashSet,
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Debug, Default, Clone)]
pub struct MockXmppService {
    pub(crate) state: Arc<RwLock<MockXmppServiceState>>,
}

#[derive(Debug)]
pub struct MockXmppServiceState {
    pub online: bool,
    pub vcards: LinkedHashMap<BareJid, VCard>,
    pub avatars: LinkedHashMap<BareJid, Option<AvatarData>>,
    pub online_members: HashSet<BareJid>,
}

impl MockXmppService {
    fn check_online(&self) -> Result<(), Error> {
        if self.state.read().unwrap().online {
            Ok(())
        } else {
            Err(Error::Other("XMPP server offline".to_owned()))?
        }
    }
}

impl Default for MockXmppServiceState {
    fn default() -> Self {
        Self {
            online: true,
            vcards: Default::default(),
            avatars: Default::default(),
            online_members: Default::default(),
        }
    }
}

impl MockXmppService {
    pub fn get_vcard(&self, jid: &BareJid) -> Result<Option<VCard>, Error> {
        self.check_online()?;

        Ok(self
            .state
            .read()
            .unwrap()
            .vcards
            .get(jid)
            .map(ToOwned::to_owned))
    }
    pub fn set_vcard(&self, jid: &BareJid, vcard: &VCard) -> Result<(), Error> {
        self.check_online()?;

        self.state
            .write()
            .unwrap()
            .vcards
            .insert(jid.to_owned(), vcard.to_owned());
        Ok(())
    }

    pub fn get_avatar(&self, jid: &BareJid) -> Result<Option<AvatarData>, Error> {
        self.check_online()?;

        trace!("Getting {jid}'s avatar…");
        Ok(self
            .state
            .read()
            .expect("`MockXmppServiceState` lock poisonned")
            .avatars
            .get(jid)
            .cloned()
            .flatten())
    }
    pub fn set_avatar(&self, jid: &BareJid, image_data: Option<AvatarData>) -> Result<(), Error> {
        self.check_online()?;

        trace!("Setting {jid}'s avatar…");
        self.state
            .write()
            .expect("`MockXmppServiceState` lock poisonned")
            .avatars
            .insert(jid.to_owned(), image_data);
        Ok(())
    }

    fn is_connected(&self, jid: &BareJid) -> Result<bool, Error> {
        self.check_online()?;

        Ok(self.state.read().unwrap().online_members.contains(jid))
    }
}

#[async_trait::async_trait]
impl XmppServiceImpl for MockXmppService {
    async fn get_vcard(
        &self,
        _ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<VCard>, Error> {
        self.get_vcard(jid)
    }
    async fn set_own_vcard(&self, ctx: &XmppServiceContext, vcard: &VCard) -> Result<(), Error> {
        self.set_vcard(&ctx.bare_jid, vcard)
    }

    async fn get_avatar(
        &self,
        _ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<AvatarData>, Error> {
        self.get_avatar(jid)
    }
    async fn set_own_avatar(
        &self,
        ctx: &XmppServiceContext,
        png_data: Vec<u8>,
    ) -> Result<(), Error> {
        self.set_avatar(
            &ctx.bare_jid,
            Some(AvatarData::Data(png_data.into_boxed_slice())),
        )
    }

    async fn is_connected(&self, _ctx: &XmppServiceContext, jid: &BareJid) -> Result<bool, Error> {
        self.is_connected(jid)
    }
}
