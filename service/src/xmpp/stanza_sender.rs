// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use minidom::Element;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::pubsub;

use super::util::PubSubQuery;

#[derive(Clone)]
pub struct StanzaSender {
    inner: Arc<Box<dyn StanzaSenderInner>>,
}

impl StanzaSender {
    pub fn new(inner: Box<dyn StanzaSenderInner>) -> Self {
        Self {
            inner: Arc::new(inner),
        }
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
            inner: Arc::new(Box::new(value)),
        }
    }
}

pub trait StanzaSenderInner: Sync + Send {
    fn send_iq(&self, iq: Iq) -> Result<Option<Element>, StanzaSenderError>;
    fn query_pubsub_node(
        &self,
        query: PubSubQuery,
    ) -> Result<Option<Vec<pubsub::Item>>, StanzaSenderError> {
        let response = self
            .send_iq(query.build())?
            .ok_or(StanzaSenderError::UnexpectedResponse)?;

        let pubsub::PubSub::Items(items) = pubsub::PubSub::try_from(response)
            .map_err(|e| StanzaSenderError::Other(format!("{e}")))?
        else {
            Err(StanzaSenderError::UnexpectedResponse)?
        };

        Ok(Some(items.items.into_iter().map(|item| item.0).collect()))
    }
}

pub type Error = StanzaSenderError;

#[derive(Debug, thiserror::Error)]
pub enum StanzaSenderError {
    #[error("`reqwest` error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Unexpected response")]
    UnexpectedResponse,
    #[error("{0}")]
    Other(String),
}
