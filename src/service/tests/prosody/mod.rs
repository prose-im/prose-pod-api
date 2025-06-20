// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::{given, then, when};
use insta::assert_snapshot;
use service::{
    prosody_config_from_db,
    server_config::{self, ServerConfig},
};

use crate::TestWorld;

#[given("nothing has changed since the initialization of the workspace")]
fn given_nothing_changed(_world: &mut TestWorld) {
    // Do nothing, even though we could performs checks
}

#[given("every optional feature has been disabled")]
async fn given_everything_disabled(world: &mut TestWorld) -> anyhow::Result<()> {
    let ref db = world.db;

    server_config::message_archive_enabled::set(db, false).await?;
    server_config::file_upload_allowed::set(db, false).await?;
    server_config::mfa_required::set(db, false).await?;
    server_config::federation_enabled::set(db, false).await?;

    Ok(())
}

#[when("generating a new Prosody configuration file from the database")]
async fn when_generating_prosody_config(world: &mut TestWorld) -> anyhow::Result<()> {
    let ref app_config = world.app_config;
    let ref dynamic_server_config =
        (server_config::get(&world.db).await).expect("Could not read server config.");
    let server_config = ServerConfig::with_default_values(dynamic_server_config, app_config);
    let prosody_config = prosody_config_from_db(server_config, app_config);
    world.prosody_config = Some(prosody_config);
    Ok(())
}

#[then(expr = "the file should match the snapshot named {string}")]
fn then_prosody_config_file_matches(world: &mut TestWorld, snapshot_name: String) {
    assert_snapshot!(snapshot_name, world.prosody_config().to_string(None))
}
