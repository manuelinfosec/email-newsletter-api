use email_newsletter_api::configuration::get_configuration;
use email_newsletter_api::configuration::Settings;
use email_newsletter_api::startup::run;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer}; // used because `tracing-subscriber` does not implement metadata inheritance
use tracing_log::LogTracer;
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
type LayeredTracing = Layered<
    BunyanFormattingLayer<fn() -> std::io::Stdout>,
    Layered<JsonStorageLayer, Layered<EnvFilter, Registry>>,
>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Printing logs from info-level and above, if `RUST_LOG` env variable is not set
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Redirect all log events to the subscriber
    LogTracer::init().expect("Failed to set logger");

    // Falling back to printing all spans from info level and above
    // if the `RUST_LOG` environment variable has not been set
    let env_filter: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Output log record in bunyan-compatible JSON-format
    let formatting_layer: BunyanFormattingLayer<fn() -> std::io::Stdout> =
        BunyanFormattingLayer::new(
            "email-newsletter-api".to_string(),
            // Use a function that returns stdout
            std::io::stdout,
        );

    // The `.with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` provided by `tracing-subscriber`
    let subscriber: LayeredTracing = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

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
