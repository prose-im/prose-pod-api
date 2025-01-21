use cucumber::World as _;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    filter,
    fmt::{
        self,
        format::{DefaultFields, FmtSpan},
    },
    layer::SubscriberExt as _,
    Layer as _,
};

#[tokio::main]
async fn main() {
    TestWorld::cucumber()
        .configure_and_init_tracing(
            fmt::layer()
                .fmt_fields(DefaultFields::new())
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
            |fmt_layer| {
                let args = std::env::args().collect::<Vec<_>>();
                let no_cucumber_spans = args.contains(&"@no-cucumber-spans".to_string());

                let targets = if no_cucumber_spans {
                    vec![("cucumber::tracing", LevelFilter::OFF)]
                } else {
                    vec![]
                };

                tracing_subscriber::registry().with(
                    filter::Targets::new()
                        .with_default(LevelFilter::TRACE)
                        .with_targets(targets)
                        .and_then(fmt_layer),
                )
            },
        )
        .run("tests/example.feature")
        .await;
}

// LIB CODE

#[tracing::instrument]
pub fn shave_yaks(n: u8) -> u8 {
    let mut shaved_yaks = 0;
    for n in 0..n {
        shave_yak(n);
        shaved_yaks += 1;
    }
    shaved_yaks
}

#[tracing::instrument]
fn shave_yak(n: u8) {
    tracing::trace!("Shaving yak {n}…");
}

// WORLD

#[derive(Debug, Default, cucumber::World)]
struct TestWorld {
    yak_count: Option<u8>,
    shaved_yaks_count: Option<u8>,
}

// STEPS

use cucumber::{given, then, when};

#[given(expr = "there are {int} yaks")]
fn given_n_yaks(world: &mut TestWorld, n: u8) {
    world.yak_count = Some(n);
}

#[when("I shave yaks")]
fn when_shave_yaks(world: &mut TestWorld) {
    let yak_count = world.yak_count.unwrap();
    world.shaved_yaks_count = Some(shave_yaks(yak_count));
}

#[then(expr = "{int} yaks are shaved")]
fn then_n_yaks_shaved(world: &mut TestWorld, n: u8) {
    assert_eq!(world.shaved_yaks_count, Some(n));
}
