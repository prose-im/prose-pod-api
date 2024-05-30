// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod notifier;
mod stanza_id_provider;
mod uuid;

pub use self::notifier::Notifier;
pub use self::stanza_id_provider::{StanzaIdProvider, UUIDStanzaIdProvider};
pub use self::uuid::Uuid;
