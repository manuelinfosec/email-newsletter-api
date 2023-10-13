use email_newsletter_api::configuration::get_configuration;
use email_newsletter_api::configuration::Settings;
use email_newsletter_api::startup::run;
use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;

// Creating migrations\20231010233108_create_subscriptions_table.sql

// Congratulations on creating your first migration!

// Did you know you can embed your migrations in your application binary?
// On startup, after creating your database connection or pool, add:

// sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?;

// Note that the compiler won't pick up new migrations if no Rust source files have changed.
// You can create a Cargo build script to work around this with `sqlx migrate build-script`.

// See: https://docs.rs/sqlx/0.5/sqlx/macro.migrate.html

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Printing logs from info-level and above, if `RUST_LOG` env variable is not set
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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

    println!("Server running at port {:?}", port);
    run(listener, connection)?.await
}
