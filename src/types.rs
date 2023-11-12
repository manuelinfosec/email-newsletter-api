// create request struct for subscription
#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}
