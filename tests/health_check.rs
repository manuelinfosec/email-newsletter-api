use email_newsletter_api::configuration::{get_configuration, DatabaseSettings, Settings};
use sqlx::PgPool;
use sqlx::{Connection, Executor, PgConnection};
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let app: TestApp = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
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
        PgConnection::connect(&config.connection_string_without_db())
            .await
            .expect("Failed to connect to Postgres.");

    // Execute database creation query with previous connection
    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to create dataabase");

    // create connection pool for test use
    let connection_pool: PgPool = PgPool::connect(&config.connection_string())
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
    // create a listener on a random port assigned by the Operating System
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");

    // retrieve port assigned by the OS
    let port = listener.local_addr().unwrap().port();

    // local address
    let address: String = format!("http://127.0.0.1:{}", port);

    // Panic if configuration cannot be read
    let mut configuration: Settings = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();

    let connection_pool: PgPool = configure_database(&configuration.database).await;

    // start the server
    let server = email_newsletter_api::startup::run(listener, connection_pool.clone())
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
    // Initialization
    let app: TestApp = spawn_app().await;
    // HTTP Client
    let client = reqwest::Client::new();

    let body = "name=Manuel&email=manuelinfosec%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    println!("Response: {response:?}");
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
