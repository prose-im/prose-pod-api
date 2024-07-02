// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod features;

use cucumber::then;

use crate::TestWorld;

#[then("the server is reconfigured")]
fn then_server_reconfigured(world: &mut TestWorld) {
    assert_ne!(world.server_ctl_state().conf_reload_count, 0);
}

#[then("the server is not reconfigured")]
fn then_server_not_reconfigured(world: &mut TestWorld) {
    assert_eq!(world.server_ctl_state().conf_reload_count, 0);
}
