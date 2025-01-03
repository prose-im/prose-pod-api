// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod cucumber_parameters;
mod features;
mod prelude;

use cucumber::World as _;
use tracing_subscriber::{
    filter::{self, LevelFilter},
    fmt::format::{self, Format},
    layer::{Layer, SubscriberExt as _},
};

use crate::{prelude::*, test_world::TestWorld};

#[tokio::main]
async fn main() {
    TestWorld::cucumber()
        // .init_tracing()
        .configure_and_init_tracing(
            format::DefaultFields::new(),
            Format::default(),
            |fmt_layer| {
                let targets = vec![
                    ("sea_orm_migration", LevelFilter::WARN),
                    ("sqlx::query", LevelFilter::ERROR),
                    ("globset", LevelFilter::WARN),
                ];

                let args = std::env::args().collect::<Vec<_>>();
                let running_few_scenarios = args.contains(&"@testing".to_owned());
                let debug = args.contains(&"@debug".to_owned());

                let default_level = if running_few_scenarios || debug {
                    LevelFilter::TRACE
                } else {
                    LevelFilter::WARN
                };

                tracing_subscriber::registry().with(
                    filter::Targets::new()
                        .with_default(default_level)
                        .with_targets(targets)
                        .and_then(fmt_layer),
                )
            },
        )
        // Fail on undefined steps
        // .fail_on_skipped()
        .run_and_exit("../../features")
        .await;
}
