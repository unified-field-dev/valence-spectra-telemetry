# valence-spectra-telemetry verification

Re-run after code or doc changes. Covered by unit + integration tests below.

## Environment

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-valence-spectra-telemetry
```

## Unit + integration (CI)

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

### TEST_MAP

| Behavior | Level | Happy | Sad | Notes |
|----------|-------|-------|-----|-------|
| `counter_delta` | unit | in-range `u64` → `i64` | overflow saturates to `i64::MAX` | `metrics::tests` |
| `sink_kind_from_env_value` | unit | `off`/`0`/`false`/`none`, `console`, `spectra`, default | unknown/empty → Spectra | `install::tests` |
| `install_from_env` | integ | `off` NoOp + default Spectra (process-isolated) | — | `OnceLock`; aliases unit-tested |
| `SpectraTelemetrySink` | integ | counter, gauge, event, event_value | unknown names / empty labels accepted | forwards via `try_*` gate |
| Typed recorders / loggers | integ | db reads/writes, denials, slow_op, error_log | empty labels / empty logger fields accepted | no Spectra sink required |
| `sink_forward` | integ | known counters + event tables | unknown name ignored; missing fields default | consumer / sink_forward |
| Topic constants | integ | `spectra.metric.*` / `spectra.event.*` with `valence_` | — | Photon wire names from codegen |

## Notes

- Process-isolated install tests cover `off` and default Spectra; env alias
  matrix is unit-tested via `sink_kind_from_env_value` because `OnceLock`
  caches the first resolution per process.
- Under Spectra `try_*` gates, sink and typed helpers may no-op when Spectra
  is unconfigured; assertions focus on contracts and non-panic forward paths
  rather than captured sink rows.
- Sad-path tests are named with `_sad` so audits detect them; they assert
  concrete saturation and acceptance defaults, beyond smoke-only checks.
