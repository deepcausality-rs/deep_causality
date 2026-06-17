## 1. Scaffolding & dependencies

- [x] 1.1 Confirm the exact Candle version to pin (resolve the design Open Question), then add `candle-core` (and `candle-nn` only if the MLP path in D2 is chosen) with default features (CPU-only) to `examples/causal_discovery_examples/Cargo.toml` `[dependencies]`.
- [x] 1.2 Add the `[[example]]` entry `name = "example_ml_rca"`, `path = "ml/ml_rca/main.rs"` to `examples/causal_discovery_examples/Cargo.toml`.
- [x] 1.3 Create `examples/causal_discovery_examples/ml/ml_rca/main.rs` with the license header and module-level `//!` docs describing the "ML detects, causality explains" pattern.
- [x] 1.4 Verify `cargo build -p causal_discovery_examples --example example_ml_rca` compiles (empty `main`), and confirm `candle-*` appears only under this example crate (`cargo tree` / manifest grep) — no `deep_causality_*` library crate gains an ML dependency.

## 2. Dataset loading (detect-stage input)

- [x] 2.1 Load `normal.csv` and `anomalous.csv` for the `carts_cpu_1` case relative to `CARGO_MANIFEST_DIR`; parse the 44 numeric feature columns and capture the header row for index→metric-name mapping.
- [x] 2.2 Build a labeled feature matrix: every `normal.csv` row labeled `0`, every `anomalous.csv` row labeled `1`; error out with the missing path on an absent/unreadable file (no partial training).
- [x] 2.3 Fit per-feature mean/standard-deviation on the training split and apply the standardizing transform; create a deterministic train/evaluation split (fixed seed).

## 3. Candle anomaly classifier (detect stage)

- [x] 3.1 Build the model in Candle (logistic regression: one linear layer + sigmoid; per D2), with fixed-seed weight initialization.
- [x] 3.2 Implement the binary-cross-entropy training loop (forward → loss → optimizer step) over the standardized training matrix.
- [x] 3.3 Implement `score(window) -> f64` returning an anomaly score in `[0, 1]`.
- [x] 3.4 Verify on the held-back split that anomalous rows receive a higher mean score than normal rows, and that scores stay within `[0, 1]`.

## 4. Causal explanation stage (reused BRCD pipeline)

- [x] 4.1 Wrap the existing CDL BRCD pipeline (`CdlConfigBuilder::build_brcd_config(...).with_brcd_config(BrcdConfig::continuous(0)).with_cpdag_path(...).build()` → `CdlBuilder::build_brcd(&config).brcd_load_input().brcd_discover().brcd_analyze().finalize()`) behind a single `explain()` call that returns the ranked culprit list.
- [x] 4.2 Confirm the wrapped pipeline reproduces the reference result (top cause `shipping_latency`, column 42) on `carts_cpu_1`.

## 5. Monadic bridge (PropagatingProcess)

- [x] 5.1 Define the `PropagatingProcess` chain: stage 1 carries the Candle anomaly score; the gate bind escalates at/above threshold and short-circuits to a "healthy" terminal value below it.
- [x] 5.2 On escalation, bind to the explain stage so the carried value becomes the root-cause verdict, and append the escalation to the `EffectLog`.
- [x] 5.3 Verify: an anomalous window escalates (verdict in carried value, escalation in `EffectLog`); a normal window short-circuits (no verdict).

## 6. Ground-truth verdict & output

- [x] 6.1 Map the BRCD top-ranked column index to a metric name via the CSV header; read the shipped `expected.txt` / `notes.txt` ground-truth ranking for the case.
- [x] 6.2 Print the end-to-end narrative: anomaly score → gate decision → ranked culprit → whether the top culprit matches the ground-truth top cause (and whether it is within top-k).

## 7. Docs & registration

- [x] 7.1 Write `examples/causal_discovery_examples/ml/ml_rca/README.md` following the standard example template (overview, what it shows, run command, expected output).
- [x] 7.2 Add an `example_ml_rca` row to `examples/causal_discovery_examples/README.md` and to the top-level `examples/README.md` causal-discovery table, including the run command.

## 8. Verification

- [x] 8.1 Run `cargo run -p causal_discovery_examples --example example_ml_rca` end to end and confirm it trains, gates, explains, and prints the ground-truth comparison without error.
- [x] 8.2 Run `cargo fmt` and `cargo clippy -p causal_discovery_examples --all-targets` clean (no warnings); confirm no library crate manifest changed.
