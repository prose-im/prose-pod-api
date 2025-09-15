// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate biscuit_auth as biscuit;

mod cucumber_parameters;
mod features;
mod prelude;

use cucumber::World as _;
use tracing_subscriber::{
    filter::{self, LevelFilter},
    fmt::format::{self, Format},
    layer::{Layer, SubscriberExt as _},
};

use crate::prelude::*;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let debug = args.iter().any(|s| s.contains("@debug"));

    if debug {
        std::env::set_var(
            "PROSE_DEBUG_USE_AT_YOUR_OWN_RISK__LOG_CONFIG_AT_STARTUP",
            "true",
        );
        std::env::set_var("PROSE_LOG__WITH_SPAN_EVENTS", "true");
        std::env::set_var("PROSE_LOG__WITH_FILE", "true");
        std::env::set_var("PROSE_LOG__WITH_LINE_NUMBER", "true");
    }

    TestWorld::cucumber()
        // .init_tracing()
        .configure_and_init_tracing(
            format::DefaultFields::new(),
            Format::default()
                .without_time()
                .with_ansi(true)
                .with_source_location(false),
            |fmt_layer| {
                let targets = vec![
                    ("sea_orm_migration", LevelFilter::WARN),
                    ("sea_orm", LevelFilter::INFO),
                    ("sqlx::query", LevelFilter::ERROR),
                    ("globset", LevelFilter::WARN),
                ];

                let default_level = if debug {
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
