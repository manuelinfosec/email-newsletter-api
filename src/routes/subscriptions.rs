use actix_web::web::{self, Form};
use actix_web::HttpResponse;
use chrono::Utc;
use sqlx::PgConnection;
use uuid::Uuid;

// create request struct for subscription
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// will always return a 200 OK
pub async fn subscribe(form: Form<FormData>, connection: web::Data<PgConnection>) -> HttpResponse {
    println!("Name: {}", form.name);
    println!("Email: {}", form.email);

    // TODO: Validate email address or throw error

    // Insert values from request to database
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_4(),
        form.email,
        form.name,
        Utc::now()
    )
    // `get_ref` is used to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`
    .execute(connection.get_ref())
    .await;
    HttpResponse::Ok().finish()
}
