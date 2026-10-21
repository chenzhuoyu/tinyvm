use tinyvm::{Unit, vm_exec};
use tracing_core::LevelFilter;
use tracing_subscriber::{
    EnvFilter, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

fn init_logs() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::OFF.into())
                .from_env()
                .expect("Cannot initialize env filter"),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_writer(std::io::stderr)
                .with_span_events(FmtSpan::FULL),
        )
        .try_init()
        .expect("Cannot initialize tracing subscriber");
}

pub fn main() -> Unit {
    init_logs();
    vm_exec()?;
    Ok(())
}
