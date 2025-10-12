// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod connection_type;
pub mod jid;

use bytes::Bytes;
pub use connection_type::*;
pub use jid::{BareJid, FullJid, JidDomain, JidNode, JID};
pub use prose_xmpp::mods::AvatarData;

impl TryFrom<AvatarData> for super::avatar::Avatar {
    type Error = super::avatar::AvatarDecodeError;

    fn try_from(avatar_data: AvatarData) -> Result<Self, Self::Error> {
        match avatar_data {
            AvatarData::Base64(base64) => Self::try_from_base64(base64),
            AvatarData::Data(data) => Self::try_from_bytes(Bytes::from(data)),
        }
    }
}
