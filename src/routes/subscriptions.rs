use actix_web::web::{self, Form};
use actix_web::HttpResponse;
use chrono::Utc;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use tracing::span::Entered;
use tracing::Instrument;
use tracing::{self, Span};
use uuid::Uuid;

// create request struct for subscription
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// will always return a 200 OK
pub async fn subscribe(form: Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id: Uuid = uuid::Uuid::new_v4();

    // tracing::info!(
    //     "Request {request_id}: Adding '{}' - '{}' as a new subscriber.",
    //     form.email,
    //     form.name
    // );

    // Spans, like logs, have an associated level
    // `info_span` creates a span at the info-level
    let request_span: Span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );

    // As long as the guard variable is not dropped, all
    // downsteams spans and log events will be registered
    // as children of the entered span.
    // Using `enter` in an async function is a reciper for disaster!
    let _request_span_guard: Entered = request_span.enter();

    // tracing::info!("Request {request_id}: Saving new subscriber details in the database");

    // TODO: Validate email address or throw error

    // `.enter` is not called on the query's span.
    // `.instrument` takes care of it at the right moment in the query's future lifetime
    let query_span: Span = tracing::info_span!(
        "Saving new subscriber details in the database",
        %request_id,
        subscriber_name = %form.name,
        subscriber_email = %form.email
    );

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
    .instrument(query_span)
    .await;

    // perform error handling on query_status
    match query_status {
        Ok(_) => {
            // log successful response
            tracing::info!("Request {request_id}: New subscriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            // log failed response
            tracing::error!("Request {request_id}: Failed to execute query: {e:?}");
            HttpResponse::InternalServerError().finish()
        }
    }

    // [...]
    // `_request_span_guard` is dropped at the end of `subscribe`
    // This is when the span is exited
}
