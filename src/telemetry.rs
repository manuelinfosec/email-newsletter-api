use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl tracing::Subscriber + Send + Sync
//     // This "weird" syntax is a higher-ranked trait bound (HRTB)
//     // It basically means that Sink implements the `MakeWriter` trait
//     // trait for all choices of the lifetime parameter `'a`
//     // Check out https://doc.rust-lang.org/nomicorn/hrtb.html
//     // for more details
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Try to retrieve the default environment filter or create a new one
    // if it doesn't exist.
    let env_filter: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter));

    // Create a Bunyan formatting layer with a closure that returns `std::io::Stdout`.
    let formatting_layer: BunyanFormattingLayer<Sink> =
        BunyanFormattingLayer::new(name.to_string(), sink);

    // Create a tracing subscriber with the specified environment filter
    // and Bunyan formatting layer.
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Initialize the LogTracer to enable integration with the `log` crate.
    tracing_log::LogTracer::init().expect("Failed to set logger");

    // Set the provided subscriber as the global default for the tracing framework.
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
