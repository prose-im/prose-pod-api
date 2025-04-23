// prose-pod-api
//
// Copyright:
//   - 2022–2025, David Bernard <david.bernard.31@gmail.com> (via <https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk>)
//   - 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)
// Inspired by: https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/f53cdecfbfe1eca6ebfb307212e5e51fc0bca677/init-tracing-opentelemetry/src/tracing_subscriber_ext.rs#L106

use init_tracing_opentelemetry::{
    tracing_subscriber_ext::{build_otel_layer, TracingGuard},
    Error,
};
use service::{app_config::LogLevel, AppConfig};
use tracing::{info, warn, Subscriber};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, EnvFilter, Layer};

// NOTE: Overriding `RUST_LOG` when building tracing filters would cause
//   `RUST_LOG` to grow infinitely, but also break dynamic log levels and leak
//   configuration on factory resets.
const LOG_LEVELS_ENV_VAR: &'static str = "PROSE_LOG";

pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // use tracing_subscriber::fmt::format::FmtSpan;
    if cfg!(debug_assertions) {
        Box::new(
            tracing_subscriber::fmt::layer()
                // .compact()
                .pretty()
                // .with_target(false)
                // .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                // .without_time()
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

/// Slight modification of
/// `init_tracing_opentelemetry::tracing_subscriber_ext::build_loglevel_filter_layer`
/// to avoid overwriting `RUST_LOG` (breaks runtime reloading of log levels).
#[must_use]
pub fn build_bootstrap_loglevel_filter_layer() -> tracing_subscriber::filter::EnvFilter {
    std::env::set_var(
        LOG_LEVELS_ENV_VAR,
        format!(
            // NOTE: `otel::tracing` must be at level info to emit OTel traces
            //   & spans.
            // NOTE: `otel::setup` must be at level debug to log detected
            //   resources and configuration read.
            "{},otel::tracing=trace",
            std::env::var("RUST_LOG")
                .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
                .unwrap_or_else(|_| "info".to_string())
        ),
    );
    EnvFilter::builder()
        .with_env_var(LOG_LEVELS_ENV_VAR)
        .from_env_lossy()
}

#[must_use]
fn build_loglevel_filter_layer(app_config: &AppConfig) -> EnvFilter {
    // NOTE: Last values take precedence in `RUST_LOG` (i.e. `trace,info` logs
    //   >`info`, while `info,trace` logs >`trace`), so important values must be
    //   added last.
    let mut rust_log: Vec<String> = vec![];

    rust_log.extend(
        match app_config.log_level {
            LogLevel::Trace => vec![
                "trace",
                "h2=info",
                "hyper_util=warn",
                "sqlx=info",
                "sea_orm::database::db_connection=info",
                "sea_orm::driver::sqlx_sqlite=info",
                "sea_orm_migration=info",
                "tonic=info",
                "tower=info",
            ],
            LogLevel::Debug => vec![
                "debug",
                "h2=info",
                "hyper_util=warn",
                "sqlx=info",
                "sea_orm::database::db_connection=info",
                "sea_orm::driver::sqlx_sqlite=info",
                "sea_orm_migration=info",
                "tonic=info",
                "tower=info",
            ],
            LogLevel::Info => vec![
                "info",
                "hyper_util=warn",
            ],
            LogLevel::Warn => vec!["warn"],
            LogLevel::Error => vec!["error"],
        }
        .into_iter()
        .map(ToOwned::to_owned),
    );

    rust_log.extend(std::env::var("RUST_LOG").ok().into_iter());

    // NOTE: `otel::tracing` must be at level info to emit OTel traces & spans.
    rust_log.push("otel::tracing=trace".to_owned());

    std::env::set_var(LOG_LEVELS_ENV_VAR, rust_log.join(","));
    EnvFilter::builder()
        .with_env_var(LOG_LEVELS_ENV_VAR)
        .from_env_lossy()
}

/// NOTE: Can be called multiple times.
pub fn update_global_log_level_filter(
    app_config: &AppConfig,
    tracing_filter_reload_handle: &tracing_subscriber::reload::Handle<EnvFilter, impl Subscriber>,
) {
    info!("Updating global log level filter…");
    tracing_filter_reload_handle
        .modify(|filter| *filter = build_loglevel_filter_layer(app_config))
        .inspect_err(|err| warn!("Error when updating global log level filter: {err}"))
        .unwrap_or_default();
}

/// Slight modification of
/// `init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers`
/// to support dynamic reloading of the log level filter.
///
/// NOTE: Can only be called once.
pub fn init_subscribers() -> Result<
    (
        TracingGuard,
        tracing_subscriber::reload::Handle<EnvFilter, impl Subscriber>,
    ),
    Error,
> {
    // Setup a temporary subscriber to log output during setup.
    let subscriber = tracing_subscriber::registry()
        .with(build_bootstrap_loglevel_filter_layer())
        .with(build_logger_text());
    let _guard = tracing::subscriber::set_default(subscriber);
    info!("init logging & tracing");

    let (otel_layer, guard) = build_otel_layer()?;

    let (loglevel_filter_layer, reload_handle) =
        tracing_subscriber::reload::Layer::new(build_bootstrap_loglevel_filter_layer());

    let subscriber = tracing_subscriber::registry()
        .with(otel_layer)
        .with(loglevel_filter_layer)
        .with(build_logger_text());
    tracing::subscriber::set_global_default(subscriber)?;
    Ok((guard, reload_handle))
}
