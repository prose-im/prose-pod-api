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
use service::{
    app_config::{self, LogFormat, LogLevel},
    AppConfig,
};
use tracing::{info, warn, Subscriber};
use tracing_subscriber::{
    fmt::{
        self,
        format::{Compact, Format, Json, Pretty},
        FormatFields, MakeWriter,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    reload, EnvFilter, Layer,
};

// NOTE: Overriding `RUST_LOG` when building tracing filters would cause
//   `RUST_LOG` to grow infinitely, but also break dynamic log levels and leak
//   configuration on factory resets.
const LOG_LEVELS_ENV_VAR: &'static str = "PROSE_LOG";

struct LogConfig {
    level: LogLevel,
    format: LogFormat,
}

impl From<&AppConfig> for LogConfig {
    fn from(app_config: &AppConfig) -> Self {
        Self {
            level: app_config.log_level,
            format: app_config.log_format,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: app_config::defaults::log_level(),
            format: app_config::defaults::log_format(),
        }
    }
}

#[must_use]
fn build_logger_layer<S>(log_config: &LogConfig) -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn with_log_format<S, N, L, T, W>(
        layer: fmt::Layer<S, N, Format<L, T>, W>,
        format: &LogFormat,
    ) -> Box<dyn Layer<S> + Send + Sync + 'static>
    where
        N: for<'writer> FormatFields<'writer> + 'static,
        fmt::Layer<S, N, Format<L, T>, W>: Layer<S> + Send + Sync + 'static,
        fmt::Layer<S, N, Format<Compact, T>, W>: Layer<S> + Send + Sync + 'static,
        fmt::Layer<S, fmt::format::JsonFields, Format<Json, T>, W>:
            Layer<S> + Send + Sync + 'static,
        fmt::Layer<S, fmt::format::Pretty, Format<Pretty, T>, W>: Layer<S> + Send + Sync + 'static,
        S: Subscriber + for<'a> LookupSpan<'a>,
        W: for<'writer> MakeWriter<'writer> + 'static,
    {
        match format {
            LogFormat::Full => layer.boxed(),
            LogFormat::Compact => layer.compact().boxed(),
            LogFormat::Json => layer.json().boxed(),
            LogFormat::Pretty => layer.pretty().boxed(),
        }
    }

    // use tracing_subscriber::fmt::format::FmtSpan;
    with_log_format(
        tracing_subscriber::fmt::layer()
            // .with_target(false)
            // .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            // .without_time()
            .with_timer(fmt::time::uptime()),
        &log_config.format,
    )
}

#[must_use]
fn build_filter_layer(log_config: &LogConfig) -> EnvFilter {
    // NOTE: Last values take precedence in `RUST_LOG` (i.e. `trace,info` logs
    //   >`info`, while `info,trace` logs >`trace`), so important values must be
    //   added last.
    let mut rust_log: Vec<String> = vec![];

    rust_log.extend(
        match log_config.level {
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

#[derive(Debug)]
pub struct TracingReloadHandles<L1, S1, L2, S2> {
    pub filter: reload::Handle<L1, S1>,
    pub logger: reload::Handle<L2, S2>,
}

impl<L1, S1, L2, S2> Clone for TracingReloadHandles<L1, S1, L2, S2> {
    fn clone(&self) -> Self {
        Self {
            filter: self.filter.clone(),
            logger: self.logger.clone(),
        }
    }
}

/// NOTE: Can be called multiple times.
pub fn update_tracing_config(
    app_config: &AppConfig,
    tracing_reload_handles: &TracingReloadHandles<
        EnvFilter,
        impl Subscriber + for<'a> LookupSpan<'a>,
        Box<dyn Layer<impl Subscriber + for<'a> LookupSpan<'a>> + Send + Sync>,
        impl Subscriber,
    >,
) {
    info!("Updating global tracing configuration…");
    let TracingReloadHandles { filter, logger } = tracing_reload_handles;
    let log_config = LogConfig::from(app_config);
    filter
        .modify(|filter| *filter = build_filter_layer(&log_config))
        .inspect_err(|err| warn!("Error when updating global log level filter: {err}"))
        .unwrap_or_default();
    logger
        .modify(|logger| *logger = build_logger_layer(&log_config))
        .inspect_err(|err| warn!("Error when updating global logger: {err}"))
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
        TracingReloadHandles<
            EnvFilter,
            impl Subscriber + for<'a> LookupSpan<'a>,
            Box<dyn Layer<impl Subscriber + for<'a> LookupSpan<'a>> + Send + Sync>,
            impl Subscriber,
        >,
    ),
    Error,
> {
    let log_config = LogConfig::default();

    // Setup a temporary subscriber to log output during setup.
    let subscriber = tracing_subscriber::registry()
        .with(build_filter_layer(&log_config))
        .with(build_logger_layer(&log_config));
    let _guard = tracing::subscriber::set_default(subscriber);
    info!("init logging & tracing");

    let (otel_layer, guard) = build_otel_layer()?;

    let (filter_layer, filter_reload_handle) =
        reload::Layer::<EnvFilter, _>::new(build_filter_layer(&log_config));
    let (logger_layer, logger_reload_handle) = reload::Layer::new(build_logger_layer(&log_config));

    let subscriber = tracing_subscriber::registry()
        .with(otel_layer)
        .with(filter_layer)
        .with(logger_layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok((
        guard,
        TracingReloadHandles {
            filter: filter_reload_handle,
            logger: logger_reload_handle,
        },
    ))
}
