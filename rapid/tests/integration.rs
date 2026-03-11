//! Integration tests for the `rapid` crate.
//!
//! Tests marked `#[ignore]` require a valid `RAPID_API_KEY` environment
//! variable and an active network connection.  Run them with:
//!
//! ```text
//! cargo test -p rapid -- --include-ignored
//! ```

use rapid::{Rapid, RapidError};

// ── Unit tests (no network) ───────────────────────────────────────────────────

#[test]
fn missing_api_key_error_display() {
    let err = RapidError::MissingApiKey(std::env::VarError::NotPresent);
    assert!(err.to_string().contains("RAPIDAPI_KEY"));
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires RAPID_API_KEY and network access"]
async fn calendar_us_with_date_range_returns_events() {
    let client = Rapid::new().expect("RAPID_API_KEY must be set");
    let events = client
        .calendar(Some("US"), Some("2024-01-01"), Some("2024-01-31"))
        .await
        .expect("calendar failed");
    assert!(!events.is_empty(), "expected at least one event");
}

#[tokio::test]
#[ignore = "requires RAPID_API_KEY and network access"]
async fn calendar_no_filter_returns_events() {
    let client = Rapid::new().expect("RAPID_API_KEY must be set");
    let events = client
        .calendar(None, None, None)
        .await
        .expect("calendar failed");
    assert!(!events.is_empty(), "expected events with no filter");
}

#[tokio::test]
#[ignore = "requires RAPID_API_KEY and network access"]
async fn calendar_event_has_title_and_date() {
    let client = Rapid::new().expect("RAPID_API_KEY must be set");
    let events = client
        .calendar(Some("US"), Some("2024-01-01"), Some("2024-01-31"))
        .await
        .expect("calendar failed");
    let has_title = events.iter().any(|e| e.title.is_some());
    let has_date = events.iter().any(|e| e.date.is_some());
    assert!(has_title, "expected at least one event with a title");
    assert!(has_date, "expected at least one event with a date");
}

#[tokio::test]
#[ignore = "requires RAPID_API_KEY and network access"]
async fn calendar_importance_in_known_range() {
    let client = Rapid::new().expect("RAPID_API_KEY must be set");
    let events = client
        .calendar(Some("US"), Some("2024-01-01"), Some("2024-01-31"))
        .await
        .expect("calendar failed");
    for e in events.iter().filter_map(|e| e.importance.as_ref()) {
        assert!(
            (1..=3).contains(e),
            "importance should be 1 (low), 2 (medium), or 3 (high), got {e}"
        );
    }
}
