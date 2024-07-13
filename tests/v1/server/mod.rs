// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod config;

use cucumber::then;
use prose_pod_api::error::Error;
use service::{
    model::ServerConfig,
    prosody_config_from_db,
    repositories::ServerConfigRepository,
    sea_orm::{prelude::*, IntoActiveModel as _},
};

use crate::TestWorld;

pub async fn given_server_config(
    world: &mut TestWorld,
    update: impl FnOnce(&mut <<ServerConfig as ModelTrait>::Entity as EntityTrait>::ActiveModel) -> (),
) -> Result<(), Error> {
    let db = world.db();
    let server_config = ServerConfigRepository::get(db)
        .await?
        .expect("Workspace should be initialized first");
    let mut model = server_config.into_active_model();
    update(&mut model);
    let server_config = model.update(db).await?;

    let app_config = &world.config;
    world.server_ctl_state_mut().applied_config =
        Some(prosody_config_from_db(server_config, app_config));

    Ok(())
}

#[then("the server is reconfigured")]
fn then_server_reconfigured(world: &mut TestWorld) {
    assert_ne!(world.server_ctl_state().conf_reload_count, 0);
}

#[then("the server is not reconfigured")]
fn then_server_not_reconfigured(world: &mut TestWorld) {
    assert_eq!(world.server_ctl_state().conf_reload_count, 0);
}
