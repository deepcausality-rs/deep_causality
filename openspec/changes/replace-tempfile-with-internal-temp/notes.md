# Notes — replace-tempfile-with-internal-temp

Machine: M3 Max, 16 cores, 128 GB. Branch `feature/replace-tempfile`, based on `9cf711e40`.

## 0.2 Baseline test counts (before any change)

Passed test cases, summed over `test result:` lines. Bazel counts include only targets that
`bazel query "tests(//<pkg>/...)"` currently defines. Stale logs from deleted targets would
otherwise inflate the total (`cfd` read 1880 before this filter).

| Crate | `cargo test -p` | `bazel test //<pkg>/...` | ignored (cargo / bazel) |
|---|---|---|---|
| `deep_causality_file` | 154 | 154 | 0 / 0 |
| `deep_causality_cfd` | 943 | 943 | 2 / — |
| `deep_causality_discovery` | 365 | 205 | 0 / 0 |

The two build systems disagree for `deep_causality_discovery`, and the difference predates this
change. `tests/config_tests.rs` is a second Cargo test binary whose whole body is `mod types;`, so
Cargo runs every `tests/types` test twice. Bazel's 205 matches the authored count: 194 `#[test]`
in `tests/`, 8 in `src/`, and 3 doc tests. This change compares each build system against its own
baseline.
