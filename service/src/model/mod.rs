// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod dns;
pub mod durations;
mod email_address;
pub(crate) mod invitations;
mod jid;
pub mod member_role;
pub mod network_checks;
pub mod pod_config;
pub mod pod_network_config;
pub mod server_config;
pub mod xmpp;

pub use durations::*;
pub use email_address::*;
pub use invitations::*;
pub use jid::*;
pub use member_role::*;
pub use network_checks::*;
pub use pod_config::*;
pub use pod_network_config::*;
pub use server_config::*;

pub use crate::entity::{
    member::Model as Member, workspace::Model as Workspace,
    workspace_invitation::Model as Invitation,
};
