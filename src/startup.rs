use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer because it isn't clonable
    let db_pool: Data<PgPool> = web::Data::new(db_pool);

    // using `move` to force the closure to take ownership of referenced variables
    let server = HttpServer::new(move || {
        App::new()
            // Register a middleware for loggin' requests
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Register the connection as part of the application's state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
