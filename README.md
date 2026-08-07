# valence-spectra-telemetry

[![CI](https://github.com/unified-field-dev/valence-spectra-telemetry/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/valence-spectra-telemetry/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[GitHub](https://github.com/unified-field-dev/valence-spectra-telemetry) · `cargo doc -p valence-spectra-telemetry --open`

Spectra telemetry for Valence [`TelemetrySink`](https://github.com/unified-field-dev/valence): DSL schemas, Photon topics, and process-global install helpers so Valence DB/ops metrics emit into Spectra.

```toml
valence-spectra-telemetry = { git = "https://github.com/unified-field-dev/valence-spectra-telemetry" }
```

```rust
use valence_spectra_telemetry::install_from_env;

// Install Spectra config/sink first, then:
let _sink = install_from_env();
// Then bootstrap the Valence router.
```

## About

- Spectra DSL sources under `schemas/` (metric inventory registered when linked; event helpers in `src/generated.rs` — see [`schemas/README.md`](schemas/README.md))
- `install_from_env()` installs Valence telemetry dispatch from env
- Metric/event names (`valence_db_reads`, `valence_slow_op`, …) and labels (`table`, `op`, `database_type`) follow Valence conventions

## Environment

| Variable | Values | Default |
|----------|--------|---------|
| `VALENCE_TELEMETRY` | `off`, `console`, `spectra` | `spectra` (when Spectra is configured) |

## Examples

Runnable smoke: [examples/README.md](examples/README.md).

## Verify

```bash
export CARGO_BUILD_JOBS=1
cargo test
```

## License

MIT. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
