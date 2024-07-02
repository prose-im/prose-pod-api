// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, ops::Deref};

use entity::{
    member,
    model::{MemberRole, JID},
};
use serde::{Deserialize, Serialize};

use crate::forms::JID as JIDUriParam;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct EnrichedMember {
    pub jid: JID,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, FromForm)]
pub struct JIDs {
    jids: Vec<JIDUriParam>,
}

impl Deref for JIDs {
    type Target = Vec<JIDUriParam>;

    fn deref(&self) -> &Self::Target {
        &self.jids
    }
}

impl Display for JIDs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.jids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
