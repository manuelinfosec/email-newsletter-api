// manual binding to address
use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Form;
use actix_web::{web, App, HttpResponse, HttpServer};

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