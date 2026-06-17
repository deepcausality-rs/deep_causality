## Context

`causal_discovery_examples` already ships `example_brcd_discovery`, which runs the CDL BRCD
root-cause pipeline on the real Sock Shop `carts_cpu_1` case (44 metrics; a `normal.csv` window and
an `anomalous.csv` window; a supplied `cpdag.csv`) and ranks `shipping_latency` (column 42) as the
top root cause, matching the case's `expected.txt`. That pipeline is the *explain* half. It has no
learned detector in front of it — the operator must already know the system is anomalous and hand it
the anomalous window.

This change adds the *detect* half with **Candle** (the HuggingFace Rust ML framework) and bridges
the two stages with a `PropagatingProcess`. The dataset is already labeled (normal rows vs anomalous
rows), which makes it a ready-made supervised training set with shipped ground truth — no new data is
needed. Candle would be the repository's first machine-learning dependency, so it must stay confined
to the example crate.

## Goals / Non-Goals

**Goals:**
- Demonstrate the Candle ↔ `deep_causality` integration pattern: a learned anomaly detector gates a
  causal root-cause explanation, sequenced through a causal monad.
- Reuse the shipped Sock Shop data, the existing CSV ingestion, and the existing CDL BRCD pipeline
  unchanged.
- Keep Candle an example-only dependency; leave all `deep_causality_*` library crates ML-free.
- Be small, fast, deterministic enough to run from `cargo run` and read like the other examples.

**Non-Goals:**
- Production model accuracy, hyperparameter tuning, cross-validation, or streaming/online training.
- GPU / feature-flagged Candle backends.
- Any change to the causal-discovery algorithms or to library crates.
- A general ML-in-`deep_causality` abstraction; this is one worked example.

## Decisions

### D1: Candle scope and features — CPU-only, example-crate-only
Add `candle-core` (and, if the model needs a layer/optimizer abstraction, `candle-nn`) under the
`causal_discovery_examples` crate's `[dependencies]` with **default features only** (no `cuda`,
`metal`, `mkl`, `accelerate`). Rationale: portability and CI build time; the example must run on any
developer machine. Alternative considered: a workspace-level optional ML feature — rejected as
over-engineering for a single example and as risking ML leaking toward library crates.

### D2: Model — logistic regression first (single linear layer + sigmoid)
The detector is a logistic regression over the 44 standardized features, trained with binary
cross-entropy by gradient descent in Candle. Rationale: ~720 rows × 44 features with a clean
normal/anomalous split is linearly separable enough; logistic regression is the smallest Candle
surface that still exercises tensors, a forward pass, a loss, and an optimizer, and it reads clearly.
Alternative considered: a one-hidden-layer MLP — kept as an optional upgrade (`candle-nn`), but not
required; it adds code without changing the integration story. The spec permits either.

### D3: Reuse the CDL BRCD pipeline verbatim for the explain stage
The explain stage calls the same public surface as `example_brcd_discovery`:
`CdlConfigBuilder::build_brcd_config().with_normal_path(..).with_anomalous_path(..)`
`.with_brcd_config(BrcdConfig::continuous(0)).with_cpdag_path(..).build()`, then
`CdlBuilder::build_brcd(&config).brcd_load_input().brcd_discover().brcd_analyze().finalize()`.
Rationale: zero new causal code, and the example demonstrably agrees with the existing reference run.
The BRCD pipeline performs its own CSV loading internally; the Candle stage loads the same files
separately for training (two read paths over the same shipped files is acceptable for an example).

### D4: Bridge with `PropagatingProcess`, not an `if`
The two stages are sequenced as a `PropagatingProcess` chain: stage 1 scores the window with Candle
and carries the anomaly score; a bind stage gates on the threshold — below threshold it
short-circuits to a "healthy" terminal value, at/above threshold it binds to stage 2, which runs the
BRCD ranking and carries the root-cause verdict forward; the escalation is appended to the
`EffectLog`. Rationale: the example's purpose is to show the integration *through the monad*, so the
control flow must be the monad, mirroring `corrective_ddos_detector` (detector-in-`State` driving an
escalation). A plain branch would defeat the point.

### D5: Standardize features on training statistics
Fit per-feature mean/standard-deviation on the training split and apply the same transform at scoring
time. Rationale: the 44 metrics span very different scales (CPU %, MB of memory, latencies); without
standardization the linear model is dominated by large-magnitude columns. Stored stats travel with
the model so scoring is consistent.

### D6: Ground-truth comparison via the CSV header and `expected.txt`
Map the BRCD top-ranked column index to a metric name using the CSV header row, then compare against
the shipped ground truth: `expected.txt` (the rank permutation; index 42 = `shipping_latency` first)
and/or `notes.txt` (the named ranking). The example prints whether the top-ranked culprit matches the
ground-truth top cause and whether it falls within the reported top-k. Rationale: makes the
"detected → explained → confirmed" narrative concrete and checkable.

### D7: Deterministic, small run
Use a fixed RNG seed for weight initialization and a deterministic train/evaluation split so the
printed scores are reproducible across runs. The default case is `carts_cpu_1` (the reference case),
matching `example_brcd_discovery`.

## Risks / Trade-offs

- **First ML dependency in the repo → build time / CI surface.** → Confine to the example crate,
  CPU-only default features; library crates stay ML-free (verified by D1's placement).
- **Tiny dataset → overfitting / run-to-run variance in training.** → Logistic regression +
  standardization + fixed seed; the example explicitly frames accuracy as illustrative, not a
  benchmark. The integration pattern, not the model, is the deliverable.
- **Two CSV read paths (Candle loads for training; BRCD loads internally).** → Acceptable for an
  example; both read the same shipped files relative to `CARGO_MANIFEST_DIR`, so they cannot diverge.
- **Candle API churn across versions.** → Pin an exact, current Candle version (see Open Questions)
  and keep the surface minimal (tensors, one linear layer, a loss, an optimizer).
- **`no_lib.rs` crate layout / example registration.** → Follow the existing `[[example]]` entries
  in `causal_discovery_examples/Cargo.toml`; the example is exempt from the coverage requirement
  (verified by running, not unit tests), per the examples policy.

## Migration Plan

Additive only. No existing example, data file, or library crate changes. Rollback is deletion of the
new example directory plus its `[[example]]` entry and the two README rows. No published artifact or
API is affected.

## Resolved Decisions (was Open Questions)

- **Candle version** — RESOLVED: use the latest `main` via a git dependency
  (`candle-core = { git = "https://github.com/huggingface/candle.git" }`), default branch.
- **`candle-core` only vs `candle-nn`** — RESOLVED: `candle-core` only. The logistic regression is
  written directly on `candle-core` — manual forward pass (`matmul` + bias + sigmoid via tensor ops),
  binary-cross-entropy loss, and a hand-rolled gradient-descent step using `loss.backward()` +
  `Var::set`. No `candle-nn` optimizer/layer abstraction is pulled in.
- **Single case or both** — RESOLVED: default to `carts_cpu_1` (the reference case), matching
  `example_brcd_discovery`. Iterating over `carts_cpu_2` is a possible later addition, not in scope.

### Explain-stage extraction (implementation note)
The CDL pipeline is run to `brcd_discover()`, yielding `CdlEffect<CDL<BrcdResults<T>>>`. `CdlEffect`
exposes a public `inner: Result<T, CdlError>`, so the ranked result is read as
`effect.inner?.state.brcd_result.ranks()` (with `.posterior()` for weights) — `BrcdResult::ranks()`
returns candidate root-cause sets best-first (top candidate = column index, e.g. 42 = `shipping_latency`).
This reuses the causal-discovery machinery and yields structured data for the gate and the
ground-truth comparison without a bespoke algorithm-layer call.
