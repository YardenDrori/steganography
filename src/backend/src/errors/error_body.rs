use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorBody {
    pub error_message: String,
}

impl ErrorBody {
    pub fn new(message: &str) -> Self {
        Self {
            error_message: message.to_string(),
        }
    }
}
