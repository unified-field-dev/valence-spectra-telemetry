//! Happy/sad coverage for `SpectraTelemetrySink`, typed helpers, `sink_forward`, and topics.
#![allow(missing_docs)]

use chrono::Utc;
use serde_json::json;
use valence_spectra_telemetry::{
    sink_forward, SpectraTelemetrySink, ValenceDbReadsRecorder, ValenceDbWritesRecorder,
    ValenceErrorLogLogger, ValencePrivacyDenialsRecorder, ValenceSlowOpLogger,
    VALENCE_DB_ERRORS_TOPIC, VALENCE_DB_READS_TOPIC, VALENCE_DB_WRITES_TOPIC,
    VALENCE_ERROR_LOG_TOPIC, VALENCE_PRIVACY_EVAL_TOPIC, VALENCE_QUERY_LOG_TOPIC,
    VALENCE_SLOW_OP_TOPIC,
};
use valence_telemetry::TelemetrySink;

#[test]
fn spectra_sink_counter_gauge_event_happy() {
    let sink = SpectraTelemetrySink::new();
    sink.record_counter(
        "valence_db_reads",
        &[("table", "users"), ("op", "get"), ("database_type", "mem")],
        1,
    );
    sink.record_gauge(
        "valence_db_wall_ms",
        &[("table", "users"), ("op", "get")],
        3.25,
    );
    sink.log_event(
        "valence_slow_op",
        &[("op", "query"), ("table", "users"), ("wall_ms", "40")],
    );
    sink.log_event_value(
        "valence_error_log",
        &json!({"op": "mutate", "table": "users", "error": "boom"}),
    );
}

#[test]
fn spectra_sink_unknown_and_empty_fields_accepted_sad() {
    let sink = SpectraTelemetrySink::new();
    // unknown metric names are dropped; empty labels on known metrics are ok
    sink.record_counter("unknown_valence_metric", &[], 0);
    sink.record_gauge("unknown_valence_gauge", &[], -1.0);
    sink.log_event("unknown_valence_event", &[]);
    sink.log_event_value("unknown_valence_event", &json!({}));
    sink.record_counter("valence_db_writes", &[], 0);
    sink.log_event("valence_query_log", &[]);
}

#[test]
fn typed_recorders_emit_without_spectra_sink_happy() {
    let ts = Utc::now();
    let labels = json!({"table": "users", "op": "get", "database_type": "mem"});
    ValenceDbReadsRecorder::record_at(1, labels.clone(), ts);
    ValenceDbWritesRecorder::record_at(1, labels.clone(), ts);
    ValencePrivacyDenialsRecorder::record_at(1, labels, ts);
    ValenceSlowOpLogger::log_at(
        "query".into(),
        "users".into(),
        "get".into(),
        "mem".into(),
        "12".into(),
        "r1".into(),
        ts,
    );
    ValenceErrorLogLogger::log_at(
        "router".into(),
        "mutate".into(),
        "users".into(),
        "mem".into(),
        "boom".into(),
        ts,
    );
}

#[test]
fn typed_recorders_empty_labels_accepted_sad() {
    let ts = Utc::now();
    ValenceDbReadsRecorder::record_at(0, json!({}), ts);
    ValenceSlowOpLogger::log_at(
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        ts,
    );
    ValenceErrorLogLogger::log_at(
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        ts,
    );
}

#[test]
fn sink_forward_known_metrics_and_events_happy() {
    let ts = Utc::now();
    let labels = json!({"table": "users", "op": "get", "database_type": "mem"});

    sink_forward::forward_counter("valence_db_reads".into(), labels.clone(), 1, ts);
    sink_forward::forward_counter("valence_db_writes".into(), labels.clone(), 1, ts);
    sink_forward::forward_counter("valence_db_errors".into(), labels.clone(), 1, ts);
    sink_forward::forward_counter("valence_privacy_denials".into(), labels, 1, ts);

    sink_forward::forward_event(
        "valence_slow_op".into(),
        json!({
            "operation": "query", "table": "users", "op": "get",
            "database_type": "mem", "wall_ms": "12", "record_id": "r1"
        }),
        ts,
    );
    sink_forward::forward_event(
        "valence_error_log".into(),
        json!({
            "source": "router", "operation": "mutate", "table": "users",
            "database_type": "mem", "message": "e"
        }),
        ts,
    );
    sink_forward::forward_event(
        "valence_privacy_eval".into(),
        json!({
            "policy": "p", "actor_kind": "user", "actor_id": "u1",
            "operation": "read", "table": "users", "rule_phase": "allow",
            "matched": "true", "check_outcome": "allow"
        }),
        ts,
    );
}

#[test]
fn sink_forward_unknown_and_missing_fields_ignored_sad() {
    let ts = Utc::now();
    // unknown metric / event ignored; missing fields default to empty strings
    sink_forward::forward_counter("not_a_valence_metric".into(), json!({}), 1, ts);
    sink_forward::forward_event("valence_query_log".into(), json!({}), ts);
    sink_forward::forward_event("unknown_table".into(), json!({}), ts);
}

#[test]
fn topic_constants_are_non_empty_happy() {
    for topic in [
        VALENCE_DB_READS_TOPIC,
        VALENCE_DB_WRITES_TOPIC,
        VALENCE_DB_ERRORS_TOPIC,
    ] {
        assert_ne!(topic, "");
        assert!(
            topic.starts_with("spectra.metric."),
            "unexpected metric topic: {topic}"
        );
        assert!(
            topic.contains("valence_"),
            "unexpected metric topic stem: {topic}"
        );
    }
    for topic in [
        VALENCE_SLOW_OP_TOPIC,
        VALENCE_ERROR_LOG_TOPIC,
        VALENCE_PRIVACY_EVAL_TOPIC,
        VALENCE_QUERY_LOG_TOPIC,
    ] {
        assert_ne!(topic, "");
        assert!(
            topic.starts_with("spectra.event."),
            "unexpected event topic: {topic}"
        );
        assert!(
            topic.contains("valence_"),
            "unexpected event topic stem: {topic}"
        );
    }
}
