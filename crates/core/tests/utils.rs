use chrono::Utc;

use superstac_core::{
    errors::{ValidationError},
    utils::{get_date_time, parse_url, validate_identifier},
};

#[test]
fn parse_url_rejects_invalid_url() {
    let result = parse_url("test-url");
    assert!(result.is_err())
}

#[test]
fn parse_url_accepts_valid_url() {
    let result = parse_url("https://example.com");
    assert!(result.is_ok())
}

#[test]
fn get_date_time_returns_current_datetime() {
    // Between the function call there'll be some milliseconds delay.
    // Using 5ms as a safe value.

    let now = Utc::now();
    let dt = get_date_time();

    let diff = (dt - now).num_milliseconds().abs();
    assert!(diff < 5);
}

#[test]
fn validate_identifier_test_cases() {
    let cases = vec![
        // identifier -> outcome
        ("osm_catalog", true),
        ("o", true),
        ("_", true),
        ("-", true),
        ("-_", true),
        ("-_test-catalog", true),
        ("-_test-_catalog", true),
        ("-_test!catalog", false),
        ("", false),
        (" ", false),
        ("  ", false),
        ("0", true),
        ("$", false),
        ("#", false),
        ("%", false),
        ("CATALOG_ID", true),
    ];

    for (case, outcome) in cases {
        if outcome {
            assert!(validate_identifier(case).is_ok())
        } else {
            assert!(validate_identifier(case).is_err())
        }
    }
}


#[test]
fn validate_identifier_rejects_missing_id() {
    let cases = vec!["", " ", "  "];

    for case in cases {
        assert!(validate_identifier(case).is_err());
        // Also assert the error message
        let err = validate_identifier(case).unwrap_err();
        assert_eq!(err, ValidationError::MissingField("id".to_owned()).into());
    }
}

#[test]
fn validate_identifier_rejects_invalid_characters() {
    let cases = vec!["$", "#", "%"];

    for case in cases {
        assert!(validate_identifier(case).is_err());
        // Also assert the error message
        let err = validate_identifier(case).unwrap_err();
        assert_eq!(
            err,
            ValidationError::InvalidIdentifier(
                "only ASCII letters, digits, hyphen, and underscore are allowed".to_owned()
            )
            .into()
        );
    }
}
