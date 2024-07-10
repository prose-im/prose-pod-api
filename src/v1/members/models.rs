// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};
use service::{controllers::member_controller, prose_xmpp::BareJid, MemberRole};

use crate::forms::JID as JIDUriParam;

type MemberModel = service::repositories::Member;

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub jid: BareJid,
    pub role: MemberRole,
}

impl From<MemberModel> for Member {
    fn from(model: MemberModel) -> Self {
        Self {
            jid: model.jid(),
            role: model.role,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnrichedMember {
    pub jid: BareJid,
    pub online: Option<bool>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

impl From<member_controller::EnrichedMember> for EnrichedMember {
    fn from(value: member_controller::EnrichedMember) -> Self {
        Self {
            jid: value.jid,
            online: value.online,
            nickname: value.nickname,
            avatar: value.avatar,
        }
    }
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
