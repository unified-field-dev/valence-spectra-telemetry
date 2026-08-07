# valence-spectra-telemetry examples

| Example | Role |
|---------|------|
| `telemetry_sink_smoke` | `install_from_env` + one typed recorder |

## 1. TelemetrySink — `telemetry_sink_smoke`

```bash
VALENCE_TELEMETRY=console CARGO_BUILD_JOBS=1 \
  cargo run -p valence-spectra-telemetry --example telemetry_sink_smoke
```

Success: stdout prints `telemetry_sink_smoke: OK`.

Install Spectra's sink first in real hosts, then this install, then build the Valence router.
