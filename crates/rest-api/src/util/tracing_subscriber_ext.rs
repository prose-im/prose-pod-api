// prose-pod-api
//
// Copyright:
//   - 2022–2025, David Bernard <david.bernard.31@gmail.com> (via <https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk>)
//   - 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)
// Inspired by: https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/f53cdecfbfe1eca6ebfb307212e5e51fc0bca677/init-tracing-opentelemetry/src/tracing_subscriber_ext.rs#L106

use init_tracing_opentelemetry::tracing_subscriber_ext::{
    build_loglevel_filter_layer, build_otel_layer, TracingGuard,
};
use tracing::{info, Subscriber};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, Layer};

pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    use tracing_subscriber::fmt::format::FmtSpan;
    if cfg!(debug_assertions) {
        Box::new(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .without_time()
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    }
}

pub fn init_subscribers() -> Result<TracingGuard, init_tracing_opentelemetry::Error> {
    //setup a temporary subscriber to log output during setup
    let subscriber = tracing_subscriber::registry()
        .with(build_loglevel_filter_layer())
        .with(build_logger_text());
    let _guard = tracing::subscriber::set_default(subscriber);
    info!("init logging & tracing");

    let (layer, guard) = build_otel_layer()?;

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(build_loglevel_filter_layer())
        .with(build_logger_text());
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(guard)
}
