//! Spectra-backed self-telemetry for [Valence]: checked-in event/metric helpers and Photon
//! topic DTOs, plus a [`TelemetrySink`](valence_telemetry::TelemetrySink) adapter that
//! forwards Valence's own runtime signals (reads/writes, slow ops, privacy evaluations,
//! deletions, …) into [Spectra].
//!
//! [Valence]'s [`TelemetrySink`](valence_telemetry::TelemetrySink) trait is deliberately
//! backend-agnostic: Valence calls `record_counter` / `record_gauge` / `log_event` /
//! `log_event_value` on whatever implementation the host installs. This crate is that
//! implementation for hosts that already emit their own telemetry through [Spectra]:
//! [`SpectraTelemetrySink`] forwards each call into `spectra-core`, and [`install_from_env`]
//! resolves and installs it process-wide based on `VALENCE_TELEMETRY`.
//!
//! [Valence]: https://github.com/unified-field-dev/valence
//! [Spectra]: https://github.com/unified-field-dev/spectra
//!
//! ## Features
//!
//! - **Env-resolved telemetry install** — Reads `VALENCE_TELEMETRY` at host boot and installs the matching
//!   process-wide `TelemetrySink` before the Valence router starts.
//!   [Get started](#env-driven-install)
//! - **Spectra `TelemetrySink` adapter** — [`SpectraTelemetrySink`] implements
//!   [`valence_telemetry::TelemetrySink`] when you wire the Spectra adapter yourself instead of
//!   using the env helper. [Get started](#direct-telemetry-sink)
//! - **Consumer-side forwarding** — [`sink_forward`] re-dispatches raw metric and event emits
//!   onto the matching typed Spectra recorder for sink consumers that re-emit Valence signals
//!   downstream. [Get started](#sink-forwarding)
//! - **Topic + codegen helpers** — Generated `*Recorder` / `*Logger` / `*Payload` / `*_TOPIC`
//!   symbols for explicit Valence telemetry emits from host or test code.
//!   [Get started](#typed-recorders)
//! - **Typed schemas** — Spectra DSL schemas for Valence's DB, privacy, and deletion tables and
//!   counters, checked in under `generated` and kept in sync with `schemas/`.
//!
//! # Getting started
//!
//! Most hosts install the telemetry sink once at startup, then build the Valence router so
//! reads, writes, and privacy-eval signals flow through Spectra automatically. Pick the env
//! helper for production hosts or wire [`SpectraTelemetrySink`] directly when tests need a
//! fixed backend.
//!
//! ## Env-driven install
//!
//! [`install_from_env`] is the default host path: it resolves `VALENCE_TELEMETRY` once at
//! process boot and registers the matching `TelemetrySink` before you build the Valence router,
//! so DB, privacy, and deletion signals flow through Spectra for the process lifetime.
//!
//! Prerequisites: Spectra must already be booted in the host process when `VALENCE_TELEMETRY` is
//! unset or set to `spectra`. Set `off` or `console` to disable or print locally.
//!
//! ```rust,no_run
//! // Call before constructing the Valence router.
//! use valence_spectra_telemetry::install_from_env;
//!
//! let sink = install_from_env();
//! let telemetry = std::env::var("VALENCE_TELEMETRY").unwrap_or_else(|_| "spectra".into());
//! assert!(!telemetry.trim().is_empty());
//! let _ = sink;
//! ```
//!
//! Runnable: `cargo run -p valence-spectra-telemetry --example telemetry_sink_smoke`.
//!
//! Next: [Direct telemetry sink](#direct-telemetry-sink) when you need explicit wiring in tests.
//!
//! ## Direct telemetry sink
//!
//! [`SpectraTelemetrySink`] is for hosts or tests that install `TelemetrySink` without reading
//! `VALENCE_TELEMETRY`. Construct the adapter and call [`valence_telemetry::install_telemetry_sink`]
//! before Valence starts emitting counters and events.
//!
//! Prerequisites: Spectra booted when using the default Spectra backend. Labels for counters and
//! gauges come from Valence callers via `TelemetrySink::record_counter` / `record_gauge` label
//! slices (`table`, `op`, `database_type`, and the other Valence schema labels).
//!
//! ```rust,no_run
//! use std::sync::Arc;
//!
//! use valence_spectra_telemetry::SpectraTelemetrySink;
//! use valence_telemetry::{install_telemetry_sink, TelemetrySink};
//!
//! let sink = SpectraTelemetrySink::new();
//! install_telemetry_sink(Arc::new(sink));
//! sink.record_counter(
//!     "valence_db_reads",
//!     &[("table", "users"), ("database_type", "sqlite")],
//!     1,
//! );
//! let metric = "valence_db_reads";
//! assert_eq!(metric, "valence_db_reads");
//! ```
//!
//! Next: [Sink forwarding](#sink-forwarding) when a Spectra sink re-emits raw Valence
//! metric names.
//!
//! ## Sink forwarding
//!
//! [`sink_forward`] maps raw Valence metric and event names onto this crate's typed
//! `*Recorder` / `*Logger` helpers. Use it from Spectra sink consumers that receive generic
//! emits and need to re-emit onto the Valence schema surface downstream.
//!
//! Prerequisites: the incoming metric or table name must match a Valence schema this crate
//! registers (`valence_db_reads`, `valence_privacy_denials`, and the other Valence topics).
//!
//! ```rust,no_run
//! use valence_spectra_telemetry::sink_forward;
//! use chrono::Utc;
//! use serde_json::json;
//!
//! sink_forward::forward_counter(
//!     "valence_db_reads".to_string(),
//!     json!({"table": "users", "database_type": "sqlite"}),
//!     1,
//!     Utc::now(),
//! );
//! let forwarded = "valence_db_reads";
//! assert_eq!(forwarded, "valence_db_reads");
//! ```
//!
//! API reference: [`sink_forward`] module. Next: [Typed recorders](#typed-recorders) when you
//! emit Valence telemetry directly without a sink hop.
//!
//! ## Typed recorders
//!
//! Generated `*Recorder` and `*Logger` types under [`helpers`] emit Valence counters and events
//! with typed labels and topic constants from [`topics`]. Call them from host code or tests when
//! you need an explicit emit instead of relying on Valence's runtime `TelemetrySink` path.
//!
//! Prerequisites: Spectra booted in the process. Import recorders from the crate root or
//! [`helpers`]; transport DTOs and `*_TOPIC` constants live in [`topics`].
//!
//! ```rust,no_run
//! use valence_spectra_telemetry::{
//!     ValenceDbReadsPayload, ValenceDbReadsRecorder, VALENCE_DB_READS_TOPIC,
//! };
//!
//! ValenceDbReadsRecorder::record(
//!     1,
//!     serde_json::json!({"table": "users", "database_type": "sqlite"}),
//! );
//! assert_eq!(ValenceDbReadsPayload::topic(), VALENCE_DB_READS_TOPIC);
//! ```
//!
//! See [`helpers`] for the full recorder/logger set and [`topics`] for transport DTOs.
//!
//! ## Environment
//!
//! | Variable | Values | Default |
//! |----------|--------|---------|
//! | `VALENCE_TELEMETRY` | `off`, `console`, `spectra` | `spectra` (when Spectra is configured) |
//!
//! # Feature flags
//!
//! This crate has no Cargo feature flags.

#![allow(clippy::too_long_first_doc_paragraph)]

mod install;
mod metrics;
mod sanitize;
mod schemas;
mod codegen {
    #![allow(
        dead_code,
        unused_imports,
        missing_docs,
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::restriction
    )]

    include!("generated.rs");
}

/// Typed emit helpers from Valence Spectra schemas.
///
/// # Examples
///
/// ```rust,no_run
/// use valence_spectra_telemetry::helpers::ValenceDbReadsRecorder;
///
/// ValenceDbReadsRecorder::record(
///     1,
///     serde_json::json!({"table": "users", "database_type": "sqlite"}),
/// );
/// ```
pub use codegen::helpers;

/// Transport `*Payload` / `*_TOPIC` DTOs from Valence Spectra schemas.
///
/// # Examples
///
/// ```rust,no_run
/// use valence_spectra_telemetry::topics::{ValenceDbReadsPayload, VALENCE_DB_READS_TOPIC};
///
/// assert_eq!(ValenceDbReadsPayload::topic(), VALENCE_DB_READS_TOPIC);
/// ```
pub use codegen::topics;

/// Forwarders for sink consumers that re-dispatch raw metric/event emits onto the matching
/// typed Spectra recorder generated from this crate's schemas.
///
/// # Examples
///
/// ```rust,no_run
/// use valence_spectra_telemetry::sink_forward;
/// use chrono::Utc;
/// use serde_json::json;
///
/// sink_forward::forward_counter(
///     "valence_db_reads".to_string(),
///     json!({"table": "users", "database_type": "sqlite"}),
///     1,
///     Utc::now(),
/// );
/// ```
pub use codegen::sink_forward;

pub use helpers::*;
pub use topics::*;

pub use install::{install_from_env, SpectraTelemetrySink};
