use chrono::{DateTime, Utc};
use url::{ParseError, Url};

use crate::errors::{SuperSTACError, ValidationError};

/// Current UTC timestamp. Centralized so tests can mock it later.
pub fn get_datetime_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Parse and validate a URL string.
pub fn parse_url(url: &str) -> Result<Url, ParseError> {
    Url::parse(url)
}

/// Validate a catalog/provider identifier. Allowed: ASCII letters, digits,
/// hyphen, underscore. Empty / whitespace-only is rejected.
pub fn validate_identifier(id: &str) -> Result<(), SuperSTACError> {
    if id.trim().is_empty() {
        return Err(ValidationError::MissingField("id".into()).into());
    }

    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(ValidationError::InvalidIdentifier(
            "only ASCII letters, digits, hyphen, and underscore are allowed".into(),
        )
        .into());
    }

    Ok(())
}
