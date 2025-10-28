use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

//we use a user id wrapper to implement for it the trait FromRequestParts that way whenever the
//struct is passed as a handler input it will authenticate the user automatically
pub struct AuthUser(i64);

impl AuthUser {
    pub fn id(&self) -> i64 {
        self.0
    }
}
