// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::{given, then, when};
use insta::assert_snapshot;
use prose_pod_core::prosody_config_from_db;
use sea_orm::*;

use crate::TestWorld;

#[given("nothing has changed since the initialization of the workspace")]
fn given_nothing_changed(_world: &mut TestWorld) {
    // Do nothing, even though we could performs checks
}

#[given("every optional feature has been disabled")]
async fn given_everything_disabled(world: &mut TestWorld) -> Result<(), DbErr> {
    let mut model = world.server_config.clone().into_active_model();

    model.message_archive_enabled = Set(false);
    model.file_upload_allowed = Set(false);
    model.mfa_required = Set(false);
    model.federation_enabled = Set(false);

    world.server_config = model.update(&world.db).await?;
    Ok(())
}

#[when("generating a new Prosody configuration file from the database")]
fn when_generating_prosody_config(world: &mut TestWorld) {
    let prosody_config = prosody_config_from_db(world.server_config.to_owned(), &world.app_config);
    world.prosody_config = Some(prosody_config);
}

#[then(expr = "the file should match the snapshot named {string}")]
fn then_prosody_config_file_matches(world: &mut TestWorld, snapshot_name: String) {
    assert_snapshot!(snapshot_name, world.prosody_config().to_string())
}
