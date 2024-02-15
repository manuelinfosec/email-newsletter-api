use email_newsletter_api::configuration::{get_configuration, DatabaseSettings, Settings};
use email_newsletter_api::telemetry;
use once_cell::sync::Lazy;

use reqwest::{Client, Response};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use sqlx::{Connection, Executor, PgConnection};
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Ensure that `tracing` stack is only initlalized once
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level: String = "info".to_string();
    let subscriber_name: String = "test".to_string();

    // We cannot assignthe output of `get_subscriber` to a variable based on the value of `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    // same type. We could work around it, but this is the most straight-forward way of moving forward.

    let test_log: Result<String, std::env::VarError> = std::env::var("TEST_LOG");

    // Read environment variable for logging
    if test_log.is_ok() && test_log.unwrap_or("false".to_string()) == "false" {
        // create tracing subscriber with output to Stdout
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);

        // register subscriber
        telemetry::init_subscriber(subscriber);
    } else {
        // create tracing subscriber with output to Void
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);

        // register subscriber
        telemetry::init_subscriber(subscriber);
    }
});

#[tokio::test]
async fn health_check_works() {
    let app: TestApp = spawn_app().await;

    let client: Client = reqwest::Client::new();

    let response: Response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create PgConnection to database
    let mut connection: PgConnection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    // Execute database creation query with previous connection
    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to create dataabase");

    // create connection pool for test use
    let connection_pool: PgPool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to create connection pool");

    // perform migrations in the `migrations` folder on the database
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    // return connection pool
    connection_pool
}

async fn spawn_app() -> TestApp {
    // Ensure that tracing subscriber is set up before further code execution...
    // ...and only invoked once, any subsequent invocations are ignored
    Lazy::force(&TRACING);

    // create a listener on a random port assigned by the Operating System
    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");

    // retrieve port assigned by the OS
    let port: u16 = listener.local_addr().unwrap().port();

    // local address
    let address: String = format!("http://127.0.0.1:{}", port);

    // Panic if configuration cannot be read
    let mut configuration: Settings = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();

    let connection_pool: PgPool = configure_database(&configuration.database).await;

    // start the server
    let server: actix_web::dev::Server =
        email_newsletter_api::startup::run(listener, connection_pool.clone())
            .expect("Failed to bind address");

    // run server as background task
    // tokio drops runtime after every test case
    // no need to implement a clean up logic due to tokio
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Initialization of TestApp
    let app: TestApp = spawn_app().await;
    // HTTP Client
    let client: Client = reqwest::Client::new();

    let body: &str = "name=Manuel&email=manuelinfosec%40gmail.com";
    let response: Response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert that 200 is returned as status code
    assert_eq!(200, response.status().as_u16());

    // Test if values saved to database
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "manuelinfosec@gmail.com");
    assert_eq!(saved.name, "Manuel")
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app: TestApp = spawn_app().await;

    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=manuel", "missing the email"),
        ("email=manuel%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // additional error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
