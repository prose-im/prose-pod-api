// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Debug, ops::Deref, sync::Arc};

use prose_xmpp::{
    mods::AvatarData, stanza::vcard4::Nickname, BareJid, ConnectionError, RequestError,
};
use secrecy::SecretString;
use tracing::debug;

pub use super::live_xmpp_service::LiveXmppService;

#[derive(Clone)]
pub struct XmppService {
    inner: Arc<XmppServiceInner>,
    ctx: XmppServiceContext,
}

impl XmppService {
    pub fn new(inner: Arc<XmppServiceInner>, ctx: XmppServiceContext) -> Self {
        Self { inner, ctx }
    }
}

impl Deref for XmppService {
    type Target = Arc<dyn XmppServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

#[derive(Clone)]
pub struct XmppServiceContext {
    pub bare_jid: BareJid,
    pub prosody_token: SecretString,
}

#[derive(Debug, Clone)]
pub struct XmppServiceInner(Arc<dyn XmppServiceImpl>);

impl XmppServiceInner {
    pub fn new(implem: Arc<dyn XmppServiceImpl>) -> Self {
        Self(implem)
    }
}

pub type VCard = prose_xmpp::stanza::VCard4;

impl XmppService {
    pub async fn get_vcard(&self, jid: &BareJid) -> Result<Option<VCard>, XmppServiceError> {
        self.deref().get_vcard(&self.ctx, jid).await
    }
    pub async fn get_own_vcard(&self) -> Result<Option<VCard>, XmppServiceError> {
        self.deref().get_own_vcard(&self.ctx).await
    }
    pub async fn set_own_vcard(&self, vcard: &VCard) -> Result<(), XmppServiceError> {
        self.deref().set_own_vcard(&self.ctx, vcard).await
    }
    pub async fn create_own_vcard(&self, name: &str) -> Result<(), XmppServiceError> {
        self.deref().create_own_vcard(&self.ctx, name).await
    }

    pub async fn get_own_nickname(&self) -> Result<Option<String>, XmppServiceError> {
        self.deref().get_own_nickname(&self.ctx).await
    }
    pub async fn set_own_nickname(&self, nickname: &str) -> Result<(), XmppServiceError> {
        self.deref().set_own_nickname(&self.ctx, nickname).await
    }

    pub async fn get_avatar(&self, jid: &BareJid) -> Result<Option<AvatarData>, XmppServiceError> {
        self.deref().get_avatar(&self.ctx, jid).await
    }
    pub async fn get_own_avatar(&self) -> Result<Option<AvatarData>, XmppServiceError> {
        self.deref().get_own_avatar(&self.ctx).await
    }
    pub async fn set_own_avatar(&self, png_data: Vec<u8>) -> Result<(), XmppServiceError> {
        self.deref().set_own_avatar(&self.ctx, png_data).await
    }

    pub async fn is_connected(&self, jid: &BareJid) -> Result<bool, XmppServiceError> {
        self.deref().is_connected(&self.ctx, jid).await
    }
}

#[async_trait::async_trait]
pub trait XmppServiceImpl: Debug + Send + Sync {
    async fn get_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<VCard>, XmppServiceError>;
    async fn get_own_vcard(
        &self,
        ctx: &XmppServiceContext,
    ) -> Result<Option<VCard>, XmppServiceError> {
        self.get_vcard(ctx, &ctx.bare_jid).await
    }
    async fn set_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        vcard: &VCard,
    ) -> Result<(), XmppServiceError>;

    async fn create_own_vcard(
        &self,
        ctx: &XmppServiceContext,
        name: &str,
    ) -> Result<(), XmppServiceError> {
        debug!("Creating {}'s vCard with name '{name}'…", ctx.bare_jid);
        let mut vcard = VCard::new();
        vcard.nickname.push(Nickname {
            value: name.to_owned(),
        });
        self.set_own_vcard(ctx, &vcard).await
    }

    async fn get_own_nickname(
        &self,
        ctx: &XmppServiceContext,
    ) -> Result<Option<String>, XmppServiceError> {
        let vcard = self.get_own_vcard(ctx).await?.unwrap_or_default();
        Ok(vcard.nickname.first().map(|v| v.value.to_owned()))
    }
    async fn set_own_nickname(
        &self,
        ctx: &XmppServiceContext,
        nickname: &str,
    ) -> Result<(), XmppServiceError> {
        debug!("Setting {}'s nickname to {nickname}…", ctx.bare_jid);
        let mut vcard = self.get_own_vcard(ctx).await?.unwrap_or_default();
        vcard.nickname = vec![Nickname {
            value: nickname.to_owned(),
        }];
        self.set_own_vcard(ctx, &vcard).await
    }

    async fn get_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<Option<AvatarData>, XmppServiceError>;
    async fn get_own_avatar(
        &self,
        ctx: &XmppServiceContext,
    ) -> Result<Option<AvatarData>, XmppServiceError> {
        self.get_avatar(ctx, &ctx.bare_jid).await
    }
    // TODO: Allow other MIME types
    // TODO: Allow setting an avatar pointing to a URL
    async fn set_own_avatar(
        &self,
        ctx: &XmppServiceContext,
        png_data: Vec<u8>,
    ) -> Result<(), XmppServiceError>;

    async fn is_connected(
        &self,
        ctx: &XmppServiceContext,
        jid: &BareJid,
    ) -> Result<bool, XmppServiceError>;
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
