// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::visualizer::demo;
use serde_json::json;

#[test]
fn additive_fields_and_unknown_components_preserve_exact_source_counts() {
    let mut snapshot = demo::snapshot(0);
    snapshot.components[0].identity.key = "future.component".into();
    snapshot.lifetime.totals.recorded_events_total.value = 9_007_199_254_740_993.into();
    let mut wire = serde_json::to_value(&snapshot).unwrap();
    wire["future_field"] = json!({"ignored": true});
    let read = decode_response(200, &serde_json::to_vec(&wire).unwrap(), false)
        .ok()
        .unwrap();
    assert_eq!(read.snapshot.components[0].identity.key, "future.component");
    assert_eq!(
        read.snapshot
            .lifetime
            .totals
            .recorded_events_total
            .value
            .get(),
        9_007_199_254_740_993
    );
}

#[test]
fn unauthorized_and_missing_are_distinct_and_invalidate_old_evidence() {
    for (reason, status, expected) in [
        (DiagnosticHttpReason::Forbidden, 403, "forbidden"),
        (DiagnosticHttpReason::Missing, 404, "Worker not found"),
        (DiagnosticHttpReason::Disabled, 503, "disabled"),
    ] {
        let body = serde_json::to_vec(&DiagnosticHttpError::new(reason)).unwrap();
        let error = decode_response(status, &body, true).err().unwrap();
        assert_eq!(error.message, expected);
        assert!(error.discard);
        assert!(!error.retryable);
    }
    let error = decode_response(
        404,
        br#"{"error":{"reason":"future_reason","secret":"do-not-display"}}"#,
        false,
    )
    .err()
    .unwrap();
    assert!(error.message.contains("unknown reason"));
    assert!(!error.message.contains("do-not-display"));
}

#[test]
fn busy_is_retryable_and_retained_snapshot_must_be_marked_stale_by_app() {
    let body = serde_json::to_vec(&DiagnosticHttpError::new(DiagnosticHttpReason::Busy)).unwrap();
    let error = decode_response(429, &body, false).err().unwrap();
    assert!(error.retryable);
    assert!(!error.discard);
}

#[test]
fn unsupported_schema_is_not_silently_interpreted() {
    let mut snapshot = serde_json::to_value(demo::snapshot(0)).unwrap();
    snapshot["schema_version"] = json!(99);
    let error = decode_response(200, &serde_json::to_vec(&snapshot).unwrap(), false)
        .err()
        .unwrap();
    assert!(error.discard);
    assert!(!error.retryable);
}

#[test]
fn worker_cache_freshness_is_independent_of_successful_http_read() {
    let snapshot = demo::snapshot(0);
    let mut page = json!({
        "assignment": {"source": snapshot.source.instance_id},
        "collection": {"availability":"stale", "latest_poll":"timeout", "age_millis":"18000"},
        "snapshot": snapshot,
    });
    let read = decode_response(200, &serde_json::to_vec(&page).unwrap(), true)
        .ok()
        .unwrap();
    let cache = read.cache.unwrap();
    assert_eq!(cache.availability, "stale");
    assert_eq!(cache.age_millis.unwrap().get(), 18000);
    page["assignment"]["source"] = json!(Uuid::nil());
    assert!(decode_response(200, &serde_json::to_vec(&page).unwrap(), true).is_err());
    page["snapshot"] = serde_json::Value::Null;
    let missing = decode_response(200, &serde_json::to_vec(&page).unwrap(), true)
        .err()
        .unwrap();
    assert!(missing.discard && missing.retryable);
}
