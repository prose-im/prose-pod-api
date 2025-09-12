// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tracing::trace;

#[derive(Debug)]
pub struct Cache<K, V> {
    data: RwLock<HashMap<K, (Instant, V)>>,
    ttl: Duration,
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub async fn get_or_insert_with<Value, F>(&self, key: &K, make: impl FnOnce() -> F) -> (V, bool)
    where
        F: Future<Output = Value>,
        Value: ToOwned<Owned = V>,
    {
        // Read the cache first.
        {
            let mut cache_guard = self.data.upgradable_read();
            if let Some((cached_at, value)) = cache_guard.get(key) {
                if cached_at.elapsed() < self.ttl {
                    trace!("Cache hit.");
                    return (value.clone(), true);
                } else {
                    // Clear the cache if it's expired.
                    cache_guard.with_upgraded(|map| map.remove(key));
                }
            };
        }

        // If there is no cached data (or expired), create a new one.
        let value = make().await;

        // Cache the new value.
        {
            self.data
                .write()
                .insert(key.to_owned(), (Instant::now(), value.to_owned()));
        }

        (value.to_owned(), false)
    }
}
