use validator::ValidationError;

/// Validates username format: alphanumeric, underscores, and hyphens only
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err(ValidationError::new("username_invalid").with_message(
            std::borrow::Cow::Borrowed(
                "Username can only contain letters, numbers, underscores, and hyphens",
            ),
        ))
    }
}
