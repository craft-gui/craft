pub struct ExampleProps {
    pub show_scrollbar: bool,
}

impl Default for ExampleProps {
    fn default() -> Self {
        ExampleProps { show_scrollbar: true }
    }
}

#[allow(dead_code)]
pub fn setup_logging() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        use tracing::level_filters::LevelFilter;
        use tracing_subscriber::fmt::format::Pretty;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::Layer;
        use tracing_web::{performance_layer, MakeWebConsoleWriter};

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .without_time()
            .with_writer(MakeWebConsoleWriter::new())
            .with_filter(LevelFilter::INFO);

        let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

        tracing_subscriber::registry().with(fmt_layer).with(perf_layer).init();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use tracing_subscriber::fmt::format::FmtSpan;
        tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).with_span_events(FmtSpan::CLOSE).init();
    }
}
