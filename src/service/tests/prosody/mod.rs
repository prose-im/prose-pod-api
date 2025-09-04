// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use cucumber::{gherkin::Step, given, then, when};
use figment::providers::Serialized;
use insta::assert_snapshot;
use jid::BareJid;
use service::{
    members::{MemberCreateForm, MemberRepository, MemberRole},
    prosody_config_from_db, server_config, AppConfig,
};

use crate::{TestWorld, CONFIG_PATH};

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

#[given(expr = "the first admin account is {string}")]
async fn given_first_admin(world: &mut TestWorld, name: String) -> anyhow::Result<()> {
    let ref db = world.db;

    MemberRepository::create(
        db,
        MemberCreateForm {
            jid: BareJid::new(&format!(
                "{name}@{domain}",
                domain = world.app_config.server_domain()
            ))
            .unwrap(),
            role: Some(MemberRole::Admin),
            joined_at: None,
            email_address: None,
        },
    )
    .await?;

    Ok(())
}

#[given("the following app configuration is set:")]
async fn given_app_config(world: &mut TestWorld, step: &Step) -> anyhow::Result<()> {
    let mut figment = AppConfig::figment_at_path(&CONFIG_PATH.clone());

    let table = step.table.as_ref().unwrap();
    // NOTE: Skip header.
    for row in table.rows.iter().skip(1) {
        let json: serde_json::Value = serde_json::from_str(&row[1])?;
        figment = figment.merge(Serialized::default(&row[0], json));
    }

    world.app_config = AppConfig::from_figment(figment).inspect_err(|err| eprintln!("{err:#}"))?;

    Ok(())
}

#[when("generating a new Prosody configuration file from the database")]
async fn when_generating_prosody_config(world: &mut TestWorld) -> anyhow::Result<()> {
    let ref app_config = world.app_config;
    let prosody_config = prosody_config_from_db(&world.db, app_config, None).await?;
    world.prosody_config = Some(prosody_config);
    Ok(())
}

#[then(expr = "the file should match the snapshot named {string}")]
fn then_prosody_config_file_matches(world: &mut TestWorld, snapshot_name: String) {
    assert_snapshot!(snapshot_name, world.prosody_config().to_string(None))
}
