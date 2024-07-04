// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod any_notifier;
mod notifier;
mod uuid;

pub use self::notifier::Notifier;
pub use self::uuid::Uuid;
