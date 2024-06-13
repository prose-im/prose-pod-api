// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};

use entity::model::JID;
use log::debug;

use crate::VCard;

use super::stanza::avatar::AvatarData;
use super::stanza::vcard::Nickname;
use super::stanza_sender;

pub struct XmppService {
    inner: XmppServiceInner,
    ctx: XmppServiceContext,
}

impl XmppService {
    pub fn new(inner: XmppServiceInner, ctx: XmppServiceContext) -> Self {
        Self { inner, ctx }
    }
}

impl Deref for XmppService {
    type Target = Mutex<dyn XmppServiceImpl>;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

pub struct XmppServiceContext {
    pub full_jid: JID,
    pub prosody_token: String,
}

#[derive(Clone)]
pub struct XmppServiceInner(Arc<Mutex<dyn XmppServiceImpl>>);

impl XmppServiceInner {
    pub fn new(implem: Arc<Mutex<dyn XmppServiceImpl>>) -> Self {
        Self(implem)
    }
}

impl XmppService {
    fn implem(&self) -> MutexGuard<dyn XmppServiceImpl + 'static> {
        self.deref().lock().unwrap()
    }

    pub fn get_vcard(&self, jid: &JID) -> Result<Option<VCard>, XmppServiceError> {
        self.implem().get_vcard(&self.ctx, jid)
    }
    pub fn set_vcard(&self, jid: &JID, vcard: &VCard) -> Result<(), XmppServiceError> {
        self.implem().set_vcard(&self.ctx, jid, vcard)
    }
    pub fn create_vcard(&self, jid: &JID, name: &str) -> Result<(), XmppServiceError> {
        self.implem().create_vcard(&self.ctx, jid, name)
    }
    pub fn set_nickname(&self, jid: &JID, nickname: &str) -> Result<(), XmppServiceError> {
        self.implem().set_nickname(&self.ctx, jid, nickname)
    }

    pub fn get_avatar(&self, jid: &JID) -> Result<Option<AvatarData>, XmppServiceError> {
        self.implem().get_avatar(&self.ctx, jid)
    }
    pub fn set_avatar(&self, jid: &JID, png_data: Vec<u8>) -> Result<(), XmppServiceError> {
        self.implem().set_avatar(&self.ctx, jid, png_data)
    }
    pub fn disable_avatar(&self, jid: &JID) -> Result<(), XmppServiceError> {
        self.implem().disable_avatar(&self.ctx, jid)
    }
}

pub trait XmppServiceImpl: Send + Sync {
    fn get_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
    ) -> Result<Option<VCard>, XmppServiceError>;
    fn set_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        vcard: &VCard,
    ) -> Result<(), XmppServiceError>;

    fn create_vcard(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        name: &str,
    ) -> Result<(), XmppServiceError> {
        let mut vcard = VCard::new();
        vcard.nickname.push(Nickname {
            value: name.to_owned(),
        });
        self.set_vcard(ctx, jid, &vcard)
    }
    fn set_nickname(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        nickname: &str,
    ) -> Result<(), XmppServiceError> {
        debug!("Setting {jid}'s nickname to {nickname}…");
        let mut vcard = self.get_vcard(ctx, jid)?.unwrap_or_default();
        vcard.nickname = vec![Nickname {
            value: nickname.to_owned(),
        }];
        self.set_vcard(ctx, jid, &vcard)
    }

    fn get_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
    ) -> Result<Option<AvatarData>, XmppServiceError>;
    // TODO: Allow other MIME types
    // TODO: Allow setting an avatar pointing to a URL
    fn set_avatar(
        &self,
        ctx: &XmppServiceContext,
        jid: &JID,
        png_data: Vec<u8>,
    ) -> Result<(), XmppServiceError>;
    fn disable_avatar(&self, ctx: &XmppServiceContext, jid: &JID) -> Result<(), XmppServiceError>;
}

pub type Error = XmppServiceError;

#[derive(Debug, thiserror::Error)]
pub enum XmppServiceError {
    #[error("Stanza error: {0}")]
    StanzaSendFailure(#[from] stanza_sender::Error),
    #[error("{0}")]
    Other(String),
}
