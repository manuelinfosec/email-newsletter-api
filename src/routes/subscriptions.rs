use actix_web::web::{self, Form};
use actix_web::HttpResponse;
use chrono::Utc;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use uuid::Uuid;

// create request struct for subscription
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// will always return a 200 OK
pub async fn subscribe(form: Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    println!("Name: {}", form.name);
    println!("Email: {}", form.email);

    // TODO: Validate email address or throw error

    // Insert values from request to database
    let query_status: Result<PgQueryResult, sqlx::Error> = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    // `get_ref` is used to get an immutable reference to the `PgPool`
    // wrapped by `web::Data`
    .execute(pool.get_ref())
    .await;

    // perform error handling on query_status
    match query_status {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
