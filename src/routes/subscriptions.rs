// manual binding to address
use actix_web::web::Form;
use actix_web::HttpResponse;

// create request struct for subscription
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// will always return a 200 OK
pub async fn subscribe(form: Form<FormData>) -> HttpResponse {
    println!("Name: {}", form.name);
    println!("Email: {}", form.email);

    HttpResponse::Ok().finish()
}
