// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use entity::model::JID;
use log::debug;
use prose_xmpp::mods::AvatarData;
use prose_xmpp::stanza::vcard::Nickname;
use prose_xmpp::{ConnectionError, RequestError};
use secrecy::Secret;

pub struct XmppService<'r> {
    inner: &'r XmppServiceInner,
    ctx: XmppServiceContext,
}

impl<'r> XmppService<'r> {
    pub fn new(inner: &'r XmppServiceInner, ctx: XmppServiceContext) -> Self {
        Self { inner, ctx }
    }
}

impl<'r> Deref for XmppService<'r> {
    type Target = Box<dyn XmppServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

pub struct XmppServiceContext {
    pub full_jid: JID,
    pub prosody_token: Secret<String>,
}

pub struct XmppServiceInner(Box<dyn XmppServiceImpl>);

impl XmppServiceInner {
    pub fn new(implem: Box<dyn XmppServiceImpl>) -> Self {
        Self(implem)
    }
}

pub type VCard = prose_xmpp::stanza::VCard4;

impl<'r> XmppService<'r> {
    pub fn get_vcard(&self, jid: &JID) -> Result<Option<VCard>, XmppServiceError> {
        self.deref().get_vcard(&self.ctx, jid)
    }
    pub fn set_own_vcard(&self, vcard: &VCard) -> Result<(), XmppServiceError> {
        self.deref().set_own_vcard(&self.ctx, vcard)
    }
    pub fn create_own_vcard(&self, name: &str) -> Result<(), XmppServiceError> {
        self.deref().create_own_vcard(&self.ctx, name)
    }
    pub fn set_own_nickname(&self, nickname: &str) -> Result<(), XmppServiceError> {
        self.deref().set_own_nickname(&self.ctx, nickname)
    }

    pub fn get_avatar(&self, jid: &JID) -> Result<Option<AvatarData>, XmppServiceError> {
        self.deref().get_avatar(&self.ctx, jid)
    }
    pub fn set_own_avatar(&self, png_data: Vec<u8>) -> Result<(), XmppServiceError> {
        self.deref().set_own_avatar(&self.ctx, png_data)
    }

    pub fn is_connected(&self, jid: &JID) -> Result<bool, XmppServiceError> {
        self.deref().is_connected(&self.ctx, jid)
    }
}

pub trait XmppServiceImpl: Send + Sync {
    fn get_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
    ) -> Result<Option<VCard>, XmppServiceError>;
    fn set_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        vcard: &VCard,
    ) -> Result<(), XmppServiceError>;

    fn create_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        name: &str,
    ) -> Result<(), XmppServiceError> {
        let mut vcard = VCard::new();
        vcard.nickname.push(Nickname {
            value: name.to_owned(),
        });
        self.set_own_vcard(ctx, &vcard)
    }
    fn set_own_nickname(
        &self,
        ctx: &XmppServiceContext,
        nickname: &str,
    ) -> Result<(), XmppServiceError> {
        debug!("Setting {}'s nickname to {nickname}…", ctx.full_jid);
        let mut vcard = self.get_vcard(ctx, &ctx.full_jid)?.unwrap_or_default();
        vcard.nickname = vec![Nickname {
            value: nickname.to_owned(),
        }];
        self.set_own_vcard(ctx, &vcard)
    }

    fn get_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
    ) -> Result<Option<AvatarData>, XmppServiceError>;
    // TODO: Allow other MIME types
    // TODO: Allow setting an avatar pointing to a URL
    fn set_own_avatar(
        &self,
        ctx: &XmppServiceContext,
        png_data: Vec<u8>,
    ) -> Result<(), XmppServiceError>;

    fn is_connected(&self, ctx: &XmppServiceContext, jid: &JID) -> Result<bool, XmppServiceError>;
}

pub type Error = XmppServiceError;

#[derive(Debug, thiserror::Error)]
pub enum XmppServiceError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("Request error: {0}")]
    RequestError(#[from] RequestError),
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for XmppServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(format!("{err}"))
    }
}
