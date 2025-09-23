// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::DatabaseConnection;

/// Separated read/write database connection pools.
///
/// This allows using different settings for both (e.g. at most one connection
/// for writing, and a higher number for concurrent reading).
///
/// This split was motivated by
/// [“database is locked” when two server configs are reset concurrently · Issue #327 · prose-im/prose-pod-api](https://github.com/prose-im/prose-pod-api/issues/327)
/// and [Database now locks more often · Issue #331 · prose-im/prose-pod-api](https://github.com/prose-im/prose-pod-api/issues/331),
/// and suggested in
/// [launchbadge/sqlx#451 (comment)](https://github.com/launchbadge/sqlx/issues/451#issuecomment-649866619).
// FIXME: Make this type-safe. I (@RemiBardon) had to go fast but this looks
//   like a huge footgun. One day we’ll have a bug because we passed a read-only
//   connection to a function that at some point tries to write something and
//   that’ll be my fault for not making this type safe from the start.
//   When we do this, also make sure the read pool is read-only. I didn’t
//   do it because we have no type safety ATM and it would just increase the
//   probability of a footgun.
#[derive(Debug, Clone)]
pub struct DatabaseRwConnectionPools {
    pub read: DatabaseConnection,
    pub write: DatabaseConnection,
}
