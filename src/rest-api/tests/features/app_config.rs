// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::prelude::app_config::reload_config;

use super::prelude::*;

// MARK: - Given

#[given(expr = "config {string} is set to {}")]
pub fn given_app_config(world: &mut TestWorld, key: String, value: String) -> anyhow::Result<()> {
    if world.api.is_some() {
        tracing::warn!("Config set after the API has started, you need to restart.");
        world.api = None;
    }
    world.set_config(&key, &value)?;
    reload_config(world);

    Ok(())
}

#[given(expr = "config {string} is unset")]
pub fn given_app_config_unset(world: &mut TestWorld, key: String) {
    if world.api.is_some() {
        tracing::warn!("Config set after the API has started, you need to restart.");
        world.api = None;
    }
    world.unset_config(&key);
    reload_config(world);
}
