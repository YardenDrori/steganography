use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

impl ErrorBody {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}
