// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use entity::model::{MemberRole, JID};

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub jid: JID,
    pub name: String,
    pub role: MemberRole,
}

// impl From<(member::Model, String)> for Member {
//     fn from(value: (member::Model, String)) -> Self {
//         let (model, domain) = value;
//         Self {
//             jid: JID {
//                 node: model.username().to_owned(),
//                 domain: domain.to_owned(),
//             },
//             name: model.name.to_owned(),
//             role: model.role,
//         }
//     }
// }
