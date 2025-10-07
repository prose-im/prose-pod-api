// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// Custom Cucumber parameters
// See <https://cucumber-rs.github.io/cucumber/current/writing/capturing.html#custom-parameters>

mod array;
mod boolean;
mod dns_record_type;
mod domain_name;
mod duration;
mod email_address;
mod http_status;
mod jid;
mod member_role;
mod open_or_not;
mod state_verb;
mod text;
mod toggle_state;

pub use array::*;
pub use boolean::*;
pub use dns_record_type::*;
pub use domain_name::*;
pub use duration::*;
pub use email_address::*;
pub use http_status::*;
pub use jid::*;
pub use member_role::*;
pub use open_or_not::*;
pub use state_verb::*;
pub use text::*;
pub use toggle_state::*;
