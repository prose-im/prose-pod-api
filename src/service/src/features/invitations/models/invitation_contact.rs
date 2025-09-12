// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serdev::{Deserialize, Serialize};

use crate::models::EmailAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// NOTE: No need to validate as `EmailAddress` is parsed.
#[serde(tag = "channel", rename_all = "snake_case")]
pub enum InvitationContact {
    Email { email_address: EmailAddress },
}
