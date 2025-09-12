// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serdev::Serialize;

use crate::{
    members::{self, MemberRole},
    models::BareJid,
};

#[derive(Debug)]
#[derive(Serialize)]
#[cfg_attr(feature = "test", derive(serdev::Deserialize))]
pub struct Member {
    pub jid: BareJid,
    pub role: MemberRole,
}

// MARK: - Boilerplate

impl From<members::entities::Member> for Member {
    fn from(model: members::entities::Member) -> Self {
        Self {
            jid: model.jid(),
            role: model.role,
        }
    }
}
