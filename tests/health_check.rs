use email_newsletter_api::configuration::{get_configuration, Settings};
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    // create a listener on a random port assigned by the Operating System
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");

    // retrieve port assigned by the OS
    let port = listener.local_addr().unwrap().port();

    // start the server
    let server = email_newsletter_api::startup::run(listener).expect("Failed to bind address");

    // run server as background task
    // tokio drops runtime after every test case
    // no need to implement a clean up logic due to tokio
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Initialization
    let app_address = spawn_app();
    let configuration: Settings = get_configuration().expect("Failed to read configuration.");
    let connection_string: String = configuration.database.connection_string();
    // The `Connection` trait MUST be in scope to invoke
    // the `PgConnection::connect` - it is not an inherent method of the struct
    let mut connection: PgConnection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to PostgreSQL database.");
    // HTTP Client
    let client = reqwest::Client::new();

    let body = "name=Manuel&email=manuelinfosec%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    println!("Response: {response:?}");
    assert_eq!(200, response.status().as_u16());

    // Test if values saved to database
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "manuelinfosec@gmail.com");
    assert_eq!(saved.name, "Manuel")
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=manuel", "missing the email"),
        ("email=manuel%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
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
