//! Install Valence `TelemetrySink` from env and emit one typed counter.
//!
//! ```bash
//! VALENCE_TELEMETRY=console CARGO_BUILD_JOBS=1 \
//!   cargo run -p valence-spectra-telemetry --example telemetry_sink_smoke
//! ```
//!
//! Success: `telemetry_sink_smoke: OK`.

#![allow(clippy::print_stdout)]

use valence_spectra_telemetry::{install_from_env, ValenceDbReadsRecorder};

fn main() {
    std::env::set_var("VALENCE_TELEMETRY", "console");
    let _sink = install_from_env();
    ValenceDbReadsRecorder::record(
        1,
        serde_json::json!({"table": "users", "database_type": "sqlite"}),
    );
    println!("telemetry_sink_smoke: OK");
}
