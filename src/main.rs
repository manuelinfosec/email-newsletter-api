use std::net::TcpListener;

use email_newsletter_api::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // in the case of an unavailable port, use port 0 which defaults to any available port
    run(listener)?.await
}
