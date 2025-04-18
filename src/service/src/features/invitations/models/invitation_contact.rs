// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::models::EmailAddress;

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "channel", rename_all = "snake_case")]
pub enum InvitationContact {
    Email { email_address: EmailAddress },
}
