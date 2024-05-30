// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use minidom::Element;
use xmpp_parsers::iq::Iq;

pub struct StanzaSender {
    inner: Box<dyn StanzaSenderInner>,
}

impl StanzaSender {
    pub fn new(inner: Box<dyn StanzaSenderInner>) -> Self {
        Self { inner }
    }
}

impl Deref for StanzaSender {
    type Target = Box<dyn StanzaSenderInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: StanzaSenderInner + 'static> From<T> for StanzaSender {
    fn from(value: T) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}

pub type R<T> = Result<T, StanzaSenderError>;

pub trait StanzaSenderInner: Sync + Send {
    fn send_iq(&self, iq: Iq) -> R<Option<Element>>;
}

pub type Error = StanzaSenderError;

#[derive(Debug, thiserror::Error)]
pub enum StanzaSenderError {
    #[error("`reqwest` error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Other(String),
}
