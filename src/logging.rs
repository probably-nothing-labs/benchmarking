use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::Layer;
use tracing_subscriber::{
    filter::FilterFn, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub const LOG_TARGET: &str = "log_target";
pub const LOG_ALL: &str = "log_all";

pub fn configure_logging() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::never(".", "output.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking_appender)
        .with_filter(FilterFn::new(|metadata| {
            metadata.target() == LOG_TARGET || metadata.target() == LOG_ALL
        }))
        .with_filter(EnvFilter::new("INFO"));

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(FilterFn::new(|metadata| metadata.target() != LOG_TARGET))
        .with_filter(EnvFilter::new("INFO"));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    _guard
}
