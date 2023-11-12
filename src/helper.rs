use chrono::Utc;
use sqlx::PgPool;

use crate::types::FormData;

#[tracing::instrument(name = "Saving new subscriber in the database", skip(form, pool))]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e: sqlx::Error| -> sqlx::Error {
        tracing::error!("Failed to execute query: {e:?}");
        e
    })?;

    Ok(())
}
