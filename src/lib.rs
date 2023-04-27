// manual binding to address
use std::net::TcpListener;

use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web::dev::Server;
use actix_web::web::Form;
use actix_web::web::Json;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

// create request struct for subscription
#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

// will always return a 200 OK
async fn subscribe(form: Form<FormData>) -> HttpResponse {
    println!("Name: {}", form.name);
    println!("Email: {}", form.email);

    HttpResponse::Ok().finish()
}


pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
        .listen(listener)?
        .run();

    Ok(server)
}