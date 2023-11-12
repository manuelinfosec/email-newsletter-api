mod configuration;
mod helper;
mod routes;
mod startup;
mod telemetry;
mod types;

use configuration::get_configuration;
use configuration::Settings;
use sqlx::PgPool;
use startup::run;
use std::net::TcpListener;

// Terminal pretty print
// cargo run | jq -R ". as $line | fromjson? // $line | del(.v, .name, .pid, .level, .hostname, .line, .time)"

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Printing logs from info-level and above, if `RUST_LOG` env variable is not set (requires `env_logger` crate)
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // use tracing's subscriber for structured logging
    let subscriber =
        telemetry::get_subscriber("email-newsletter-api".into(), "info".into(), || {
            std::io::stdout()
        });

    // initialize subscriber globally
    telemetry::init_subscriber(subscriber);

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
