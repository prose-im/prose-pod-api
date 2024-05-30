// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use entity::{
    member,
    model::{MemberRole, JID},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub jid: JID,
    pub role: MemberRole,
}

impl From<member::Model> for Member {
    fn from(model: member::Model) -> Self {
        Self {
            jid: model.jid(),
            role: model.role,
        }
    }
}
