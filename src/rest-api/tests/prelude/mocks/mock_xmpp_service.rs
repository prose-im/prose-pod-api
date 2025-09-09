// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use linked_hash_map::LinkedHashMap;
use service::{
    models::{Avatar, AvatarOwned},
    xmpp::{xmpp_service::Error, BareJid, VCard, XmppServiceContext, XmppServiceImpl},
};
use tracing::{instrument, trace};

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
    pub avatars: LinkedHashMap<BareJid, Option<AvatarOwned>>,
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
    #[instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
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
    #[instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    pub fn set_vcard(&self, jid: &BareJid, vcard: &VCard) -> Result<(), Error> {
        self.check_online()?;

        self.state
            .write()
            .unwrap()
            .vcards
            .insert(jid.to_owned(), vcard.to_owned());
        Ok(())
    }

    #[instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    pub fn get_avatar<'a>(&'a self, jid: &BareJid) -> Result<Option<AvatarOwned>, Error> {
        self.check_online()?;

        trace!("Getting {jid}'s avatar…");
        let state = (self.state.read()).expect("`MockXmppServiceState` lock poisonned");
        Ok(state.avatars.get(jid).cloned().flatten())
    }
    #[instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    pub fn set_avatar<'a>(&self, jid: &BareJid, avatar: Option<Avatar<'a>>) -> Result<(), Error> {
        self.check_online()?;

        trace!("Setting {jid}'s avatar…");
        self.state
            .write()
            .expect("`MockXmppServiceState` lock poisonned")
            .avatars
            .insert(jid.to_owned(), avatar.map(|a| a.to_owned()));
        Ok(())
    }

    #[instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
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

    async fn get_avatar<'a>(
        &'a self,
        _ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<AvatarOwned>, Error> {
        self.get_avatar(jid)
    }
    async fn set_own_avatar<'a>(
        &self,
        ctx: &XmppServiceContext,
        avatar: Avatar<'a>,
    ) -> Result<(), Error> {
        self.set_avatar(&ctx.bare_jid, Some(avatar))
    }

    async fn is_connected(&self, _ctx: &XmppServiceContext, jid: &BareJid) -> Result<bool, Error> {
        self.is_connected(jid)
    }
}
