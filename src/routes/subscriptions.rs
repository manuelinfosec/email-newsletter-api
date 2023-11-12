use crate::helper;
use crate::types::FormData;

use actix_web::web::{self, Form};
use actix_web::HttpResponse;
use chrono::Utc;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use tracing::Instrument;
use tracing::{self, Span};
use uuid::Uuid;

#[tracing::instrument(name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(form: Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    // TODO: Validate email address or throw error

    // `.enter` is not called on the query's span.
    // `.instrument` takes care of it at the right moment in the query's future lifetime
    let query_span: Span = tracing::info_span!("Saving new subscriber details in the database",);

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
            tracing::info!("New subscriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            // log failed response
            tracing::error!("Failed to execute query: {e:?}");
            HttpResponse::InternalServerError().finish()
        }
    }

    // [...]
    // `_request_span_guard` is dropped at the end of `subscribe`
    // This is when the span is exited
}
