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
    fmt::{
        self,
        format::{DefaultFields, FmtSpan, Format},
    },
    layer::{Layer, SubscriberExt as _},
};

use crate::{prelude::*, test_world::TestWorld};

#[tokio::main]
async fn main() {
    TestWorld::cucumber()
        // .init_tracing()
        .configure_and_init_tracing(
            fmt::layer()
                .fmt_fields(DefaultFields::new())
                .event_format(
                    Format::default()
                        .without_time()
                        .with_ansi(true)
                        .with_source_location(false),
                )
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
            |fmt_layer| {
                let targets = vec![
                    ("sea_orm_migration", LevelFilter::WARN),
                    ("sea_orm", LevelFilter::INFO),
                    ("sqlx::query", LevelFilter::ERROR),
                    ("globset", LevelFilter::WARN),
                    ("cucumber::tracing", LevelFilter::OFF),
                ];

                let args = std::env::args().collect::<Vec<_>>();
                let debug = args.iter().any(|s| s.contains("@debug"));

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
