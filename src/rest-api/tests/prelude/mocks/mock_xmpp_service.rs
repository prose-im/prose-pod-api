// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{
    models::{Avatar, AvatarOwned},
    xmpp::{xmpp_service::Error, VCard, XmppServiceContext, XmppServiceImpl},
};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MockXmppService {
    pub(crate) state: Arc<RwLock<MockXmppServiceState>>,
    pub mock_server_state: Arc<RwLock<MockServerServiceState>>,
}

#[derive(Debug, Default)]
pub struct MockXmppServiceState {
    pub vcards: LinkedHashMap<BareJid, VCard>,
    pub avatars: LinkedHashMap<BareJid, Option<AvatarOwned>>,
    pub online_members: HashSet<BareJid>,
}

impl MockXmppService {
    #[tracing::instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    pub fn get_vcard(&self, jid: &BareJid) -> Result<Option<VCard>, Error> {
        check_online(&self.mock_server_state)?;

        Ok(self
            .state
            .read()
            .unwrap()
            .vcards
            .get(jid)
            .map(ToOwned::to_owned))
    }

    #[tracing::instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    pub fn set_vcard(&self, jid: &BareJid, vcard: &VCard) -> Result<(), Error> {
        check_online(&self.mock_server_state)?;

        Self::set_vcard_(&self.state, jid, vcard);

        Ok(())
    }

    #[tracing::instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    pub fn get_avatar<'a>(&'a self, jid: &BareJid) -> Result<Option<AvatarOwned>, Error> {
        check_online(&self.mock_server_state)?;

        tracing::trace!("Getting {jid}'s avatar…");
        let state = (self.state.read()).expect("`MockXmppServiceState` lock poisonned");
        Ok(state.avatars.get(jid).cloned().flatten())
    }

    #[tracing::instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    pub fn set_avatar<'a>(&self, jid: &BareJid, avatar: Option<Avatar<'a>>) -> Result<(), Error> {
        check_online(&self.mock_server_state)?;

        tracing::trace!("Setting {jid}'s avatar…");
        self.state
            .write()
            .expect("`MockXmppServiceState` lock poisonned")
            .avatars
            .insert(jid.to_owned(), avatar.map(|a| a.to_owned()));
        Ok(())
    }

    #[tracing::instrument(
        level = "trace",
        skip_all, fields(jid = jid.to_string()),
        ret(level = "trace"), err(level = "trace")
    )]
    fn is_connected(&self, jid: &BareJid) -> Result<bool, Error> {
        check_online(&self.mock_server_state)?;

        Ok(self.state.read().unwrap().online_members.contains(jid))
    }
}

impl MockXmppService {
    pub(crate) fn set_vcard_(
        state: &Arc<RwLock<MockXmppServiceState>>,
        jid: &BareJid,
        vcard: &VCard,
    ) {
        state
            .write()
            .unwrap()
            .vcards
            .insert(jid.to_owned(), vcard.to_owned());
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
