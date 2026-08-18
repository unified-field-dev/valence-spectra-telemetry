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
//! Typed [`helpers`] (`*Logger` / `*Recorder`), [`topics`] (`*Payload` / `*_TOPIC` DTOs),
//! and [`sink_forward`] live in the checked-in `generated` module (formerly produced by
//! `spectra-codegen`). DSL sources under `schemas/` remain the human-edited schema truth.
//!
//! [Valence]: https://github.com/unified-field-dev/valence
//! [Spectra]: https://github.com/unified-field-dev/spectra
//!
//! ## Features
//!
//! - **`TelemetrySink` install** — [`SpectraTelemetrySink`] implements
//!   [`valence_telemetry::TelemetrySink`] by routing counters, gauges, and events through
//!   `spectra-core`.
//! - **Env-driven install** — [`install_from_env`] reads `VALENCE_TELEMETRY`
//!   (`off` / `console` / default-to-Spectra) and installs the matching sink process-wide.
//! - **Checked-in typed helpers** — Spectra schema helpers and topic DTOs under `generated`,
//!   kept in sync with the DSL sources in `schemas/`.
//! - **Topic + helpers** — `*Payload` / `*_TOPIC` DTOs and `*Recorder` / `*Logger` types,
//!   importable straight from the crate root.
//! - **Consumer-side forwarding** — [`sink_forward`] re-dispatches raw metric/event emits onto
//!   the matching typed Spectra recorder, for sink consumers that re-emit Valence's signals
//!   downstream instead of calling the typed helpers directly.
//!

//!
//! ## Concern → API
//!
//! | Concern | API |
//! |---|---|
//! | Install | [`install_from_env`] / [`SpectraTelemetrySink`] |
//! | Sink forwarding | [`sink_forward`] |
//!
//! Labels (`table` / `op` / `database_type`) are supplied by Valence itself; this crate has no
//! dedicated label types.
//!
//! ## Generated schemas & topics
//!
//! Typed `*Recorder` / `*Logger` / `*Payload` / `*_TOPIC` symbols are re-exported at the crate
//! root and grouped under [`helpers`] and [`topics`]. One mid-level pattern for both surfaces:
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
//! # Getting started
//!
//! Install Spectra's own sink first, then install and hand this crate's sink to your Valence
//! router bootstrap:
//!
//! ```rust,no_run
//! use valence_spectra_telemetry::install_from_env;
//!
//! // Reads `VALENCE_TELEMETRY` (off / console / default-to-Spectra), installing the
//! // resolved sink as Valence's process-global telemetry dispatch target.
//! let _sink = install_from_env();
//!
//! // ... build your Valence router / bootstrap; Valence's own reads, writes, and
//! // privacy-eval events now flow through Spectra automatically.
//! ```
//!
//! Runnable: `cargo run -p valence-spectra-telemetry --example telemetry_sink_smoke`.
//!
//! ## Where to look next
//!
//! - [`install_from_env`] / [`SpectraTelemetrySink`] — process-wide `TelemetrySink` bootstrap
//! - [`sink_forward`] — forwarders for sink consumers that re-emit onto the typed Spectra recorders
//! - [`helpers`] / [`topics`] — generated recorders, loggers, payloads, and topic constants
//! - [`ValenceSlowOpLogger`] / [`ValenceDbReadsRecorder`] — representative event/metric helpers

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
