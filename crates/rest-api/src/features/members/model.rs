// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};
use service::{model::MemberRole, prose_xmpp::BareJid};

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub jid: BareJid,
    pub role: MemberRole,
}

// BOILERPLATE

impl From<service::model::Member> for Member {
    fn from(model: service::model::Member) -> Self {
        Self {
            jid: model.jid(),
            role: model.role,
        }
    }
}
