use email_newsletter_api::configuration::get_configuration;
use email_newsletter_api::configuration::Settings;
use email_newsletter_api::startup::run;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer}; // used because `tracing-subscriber` does not implement metadata inheritance
use tracing_subscriber::layer::Layered;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// Creating migrations\20231010233108_create_subscriptions_table.sql

// Congratulations on creating your first migration!

// Did you know you can embed your migrations in your application binary?
// On startup, after creating your database connection or pool, add:

// sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?;

// Note that the compiler won't pick up new migrations if no Rust source files have changed.
// You can create a Cargo build script to work around this with `sqlx migrate build-script`.

// See: https://docs.rs/sqlx/0.5/sqlx/macro.migrate.html

// Custom type for nested annotations
// type LayeredTracing = Layered<
//     BunyanFormattingLayer<fn() -> std::io::Stdout>,
//     Layered<JsonStorageLayer, Layered<EnvFilter, Registry>>,
// >;

/// Constructs and returns a tracing subscriber suitable for use in a tracing-enabled
/// Rust application. The subscriber is configured with an environment filter
/// and a Bunyan formatting layer for structured logging.
///
/// # Arguments
///
/// * `name`: A name associated with the application or service. This will be used
///   for logging purposes.
/// * `env_filter`: A string specifying the environment filter for tracing logs.
///
/// # Returns
///
/// An implementation of the `Subscriber` trait combined with `Send` and `Sync` traits.
///
/// # Examples
///
/// ```rust
/// use tracing::subscriber::Subscriber;
/// use tracing_subscriber::{Registry, EnvFilter, BunyanFormattingLayer};
///
/// // Get a tracing subscriber with a custom name and environment filter.
/// let subscriber = get_subscriber("my-app".to_string(), "info".to_string());
///
/// // Use the subscriber in your application.
/// // ...
/// ```
///
/// # Panics
///
/// This function panics if it fails to retrieve the default environment filter or
/// if there are issues setting up the Bunyan formatting layer.
pub fn get_subscriber(name: String, env_filter: String) -> impl tracing::Subscriber + Send + Sync {
    // Try to retrieve the default environment filter or create a new one
    // if it doesn't exist.
    let env_filter: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter));

    // Create a Bunyan formatting layer with a closure that returns `std::io::Stdout`.
    let formatting_layer: BunyanFormattingLayer<fn() -> std::io::Stdout> =
        BunyanFormattingLayer::new(name.to_string(), || std::io::stdout());

    // Create a tracing subscriber with the specified environment filter
    // and Bunyan formatting layer.
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Initializes the global subscriber for the tracing framework and sets up
/// logging using the [`tracing_log`] crate.
///
/// # Arguments
///
/// * `subscriber`: An implementation of the `Subscriber` trait combined with
///   `Send` and `Sync` traits. This subscriber will be set as the global default
///   for the tracing framework.
///
/// # Panics
///
/// This function panics if it fails to set up the logger or set the subscriber
/// as the global default.
///
/// # Examples
///
/// ```rust
/// use tracing::subscriber::Subscriber;
/// use tracing_subscriber::{Registry, EnvFilter, BunyanFormattingLayer};
/// use tracing_log::LogTracer;
///
/// // Define a custom subscriber, e.g., using the `Registry` and
/// // `BunyanFormattingLayer`.
/// let subscriber = Registry::default()
///     .with(EnvFilter::new("info"))
///     .with(BunyanFormattingLayer::new("my-app".to_string(), || Box::new(std::io::stdout())));
///
/// // Initialize the subscriber and set up logging.
/// init_subscriber(subscriber);
/// ```
///
/// [`tracing_log`]: https://docs.rs/tracing-log
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Initialize the LogTracer to enable integration with the `log` crate.
    tracing_log::LogTracer::init().expect("Failed to set logger");

    // Set the provided subscriber as the global default for the tracing framework.
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Printing logs from info-level and above, if `RUST_LOG` env variable is not set
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let subscriber = get_subscriber("email-newsletter-api".into(), "info".into());
    init_subscriber(subscriber);

    // Panic if configuration cannot be read
    let configuration: Settings = get_configuration().expect("Failed to read configuration");

    // Create PgConnection to database | `sqlx::PgPool` is type alias for `sqlx::Pool<sqlx::Postgress>`
    let connection: PgPool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // Create address string from configuration file
    let address: String = format!("127.0.0.1:{}", configuration.application_port);

    // in the case of an unavailable port, use port 0 which defaults to any available port assigned by the OS
    let listener: TcpListener = TcpListener::bind(address).expect("Failed to bind to random port");

    // collect the port of the listener
    let port: u16 = listener.local_addr().unwrap().port();

    println!(r#"{{"msg": "Server Running at {:?}"}}"#, port);
    run(listener, connection)?.await
}
