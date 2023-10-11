use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::PgConnection;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, connection: PgConnection) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer because it isn't clonable
    let connection: Data<PgConnection> = web::Data::new(connection);

    // using `move` to force the closure to take ownership of referenced variables
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Register the connection as part of the application's state
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
