#[allow(dead_code)]
fn main() {
}

pub fn setup_logging() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        use tracing_web::{MakeWebConsoleWriter, performance_layer};
        use tracing_subscriber::fmt::format::Pretty;
        use tracing::level_filters::LevelFilter;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::Layer;

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .without_time()
            .with_writer(MakeWebConsoleWriter::new())
            .with_filter(LevelFilter::INFO);
        
        let perf_layer = performance_layer()
            .with_details_from_fields(Pretty::default());

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(perf_layer)
            .init();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing_subscriber::fmt()
            .without_time()
            .with_max_level(tracing::Level::INFO)
            .init();
    }
}