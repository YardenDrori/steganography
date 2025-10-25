use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorBody {
    pub error_message: String,
}
