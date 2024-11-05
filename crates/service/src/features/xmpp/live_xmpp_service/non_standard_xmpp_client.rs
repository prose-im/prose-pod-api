// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;

use async_trait::async_trait;

use crate::model::BareJid;

/// Sometimes there are actions the XMPP protocol doesn't support, like querying a user's presence.
/// By discussing directly with the XMPP server, we can still get this information.
/// This trait contains all the methods we'd need in `XmppClient` but can't support there.
#[async_trait]
pub trait NonStandardXmppClient: Debug + Send + Sync {
    async fn is_connected(&self, jid: &BareJid) -> Result<bool, anyhow::Error>;
}
