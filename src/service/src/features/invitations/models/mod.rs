// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod invitation_channel;
mod invitation_contact;
mod invitation_status;
mod invitation_token_type;
mod workspace_invitation_notification;

pub use super::entities::workspace_invitation::Model as Invitation;

pub use self::invitation_channel::*;
pub use self::invitation_contact::*;
pub use self::invitation_status::*;
pub use self::invitation_token_type::*;
pub use self::workspace_invitation_notification::*;

pub type InvitationId = i32;
