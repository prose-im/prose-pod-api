// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};

use entity::model::JID;

use crate::VCard;

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
    pub bare_jid: JID,
}

#[derive(Clone)]
pub struct XmppServiceInner(Arc<Mutex<dyn XmppServiceImpl>>);

impl XmppServiceInner {
    pub fn new(implem: Arc<Mutex<dyn XmppServiceImpl>>) -> Self {
        Self(implem)
    }
}

pub type R<T> = Result<T, XmppServiceError>;

impl XmppService {
    fn implem(&self) -> MutexGuard<dyn XmppServiceImpl + 'static> {
        self.deref().lock().unwrap()
    }

    pub fn get_vcard(&self, jid: &JID) -> R<Option<VCard>> {
        self.implem().get_vcard(&self.ctx, jid)
    }
    pub fn set_vcard(&self, jid: &JID, vcard: &VCard) -> R<()> {
        self.implem().set_vcard(&self.ctx, jid, vcard)
    }
    pub fn create_vcard(&self, jid: &JID, name: &str) -> R<()> {
        self.implem().create_vcard(&self.ctx, jid, name)
    }
    pub fn set_nickname(&self, jid: &JID, nickname: &str) -> R<()> {
        self.implem().set_nickname(&self.ctx, jid, nickname)
    }
}

pub trait XmppServiceImpl: Send + Sync {
    fn get_vcard(&self, ctx: &XmppServiceContext, jid: &JID) -> R<Option<VCard>>;
    fn set_vcard(&self, ctx: &XmppServiceContext, jid: &JID, vcard: &VCard) -> R<()>;

    fn create_vcard(&self, ctx: &XmppServiceContext, jid: &JID, name: &str) -> R<()> {
        let mut vcard = VCard::new();
        vcard.nickname.push(Nickname {
            value: name.to_owned(),
        });
        self.set_vcard(ctx, jid, &vcard)
    }
    fn set_nickname(&self, ctx: &XmppServiceContext, jid: &JID, nickname: &str) -> R<()> {
        let mut vcard = self.get_vcard(ctx, jid)?.unwrap_or_default();
        vcard.nickname = vec![Nickname {
            value: nickname.to_owned(),
        }];
        self.set_vcard(ctx, jid, &vcard)
    }
}

pub type Error = XmppServiceError;

#[derive(Debug, thiserror::Error)]
pub enum XmppServiceError {
    #[error("Could not send stanza: {0}")]
    StanzaSendFailure(#[from] stanza_sender::Error),
    #[error("{0}")]
    Other(String),
}
