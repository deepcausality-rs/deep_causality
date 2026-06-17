# ML-gated Causal Root-Cause Analysis (Candle × DeepCausality)

**"ML detects, causality explains."** This example combines the [Candle](https://github.com/huggingface/candle)
machine-learning framework with DeepCausality on a problem each one solves only halfway:

- A small **Candle** classifier learns, from labeled telemetry, to *detect* that a microservice
  system is anomalous. It is fast and cheap, yet it cannot say *which* service is at fault.
- The existing DeepCausality **BRCD** causal-discovery pipeline *explains* the anomaly by ranking the
  culprit service or metric, but it needs a detector to decide *when* to run.

The two stages are sequenced through a `PropagatingProcess` monad. The Candle anomaly score **gates**
the causal stage, the carried value becomes the root-cause verdict, and the escalation is recorded in
the `EffectLog`. This is the canonical AIOps "detect, triage, root-cause" split, expressed as one
typed, auditable causal chain.

```text
 telemetry row ──► Candle detector ──► anomaly score ──► gate (PropagatingProcess.bind)
                    (logistic reg.)                         │
                                          score < θ ────────┤───► Healthy   (short-circuit, no RCA)
                                          score ≥ θ ────────┘───► BRCD causal RCA ──► ranked culprit
                                                                          │
                                                                          └─► checked vs ground truth
```

## Why both, and why it is not redundant

A classifier reduces 44 noisy signals to a single "is something wrong?" probability. A positive is an
alarm, not a diagnosis. The causal pipeline consumes the *normal* and *anomalous* windows plus the
service-call graph (CPDAG) and returns a *ranked, posterior-weighted* set of candidate root causes.
Running the more expensive causal stage only when the learned detector fires is how you would wire an
on-call pipeline: cheap detection everywhere, causal explanation on escalation.

## Architecture (three files)

| File | Responsibility |
|------|----------------|
| [`model.rs`](model.rs) | The value/state/context types (`RcaSignal`, `RcaState`, `RcaConfig`), the Candle `Detector` (training and scoring), the reused BRCD explainer, and the `PropagatingProcess` **gate** that bridges detect to explain. |
| [`utils.rs`](utils.rs) | Dataset loading (`load_csv`, `load_truth_index`), preparation (`build_training_set`, `fit_standardizer`), and all console-reporting functions. |
| [`main.rs`](main.rs) | A lean orchestration: load, train, calibrate the gate, run both windows, print. |

### The detector (Candle, `candle-core` only)

A logistic regression written directly on `candle-core`, with **no `candle-nn`**:

- forward pass: `x.matmul(w) + b`, then a `sigmoid` built from `affine`/`exp`/`recip`;
- binary-cross-entropy loss over the standardized feature matrix;
- a **hand-rolled gradient-descent step**: `loss.backward()`, then `GradStore::get` and `Var::set`.

Weights are zero-initialized and the split is fixed, so the run is deterministic.

### The explainer (DeepCausality BRCD)

Reuses the exact CDL surface as [`example_brcd_discovery`](../../cdl/brcd_discovery/main.rs):
`CdlConfigBuilder::build_brcd_config(...)` feeds `CdlBuilder::build_brcd(&cfg).brcd_load_input().brcd_discover()`,
and the top-ranked culprit column is read from `BrcdResult::ranks()` and `::posterior()`.

### The bridge (`PropagatingProcess`)

A single `bind` stage gates on the score. Below threshold it short-circuits to `Healthy` and the
causal stage never runs. At or above threshold it escalates, runs BRCD, compares the culprit to
ground truth, and carries a `RootCause` verdict. Every decision is appended to the process
`EffectLog`.

## Data

The shipped RCAEval Sock Shop case [`data/sock-shop-2/carts_cpu_1/`](../../data/sock-shop-2/carts_cpu_1/):

- `normal.csv` (label 0) and `anomalous.csv` (label 1): 44 service metrics (CPU, memory, workload,
  latency), about 720 labeled rows in total. This is a ready-made supervised training set.
- `expected.txt`: the ground-truth root-cause ranking; `shipping_latency` (column 42) is first.
- `cpdag.csv`: the supplied service-call causal graph the BRCD stage consumes.

No new data is added. Candle is an **example-only** dependency (latest `main`, CPU-only); no
`deep_causality_*` library crate depends on it.

## Run

```bash
cargo run -p causal_discovery_examples --example example_ml_rca
```

## Expected output (abridged)

```text
Ground-truth root cause: shipping_latency (col 42).

Candle detector, held-out mean anomaly score:
  normal window    : 0.569
  anomalous window : 1.000
Gate threshold (calibrated midpoint): 0.784

--- Window: normal-window ---
  verdict: HEALTHY (score 0.569); causal stage not run.

--- Window: anomalous-window ---
  verdict: ROOT CAUSE = shipping_latency (col 42, posterior 1.0000)
  ground-truth check: MATCH ✓ (ML detected, causality explained, ground truth confirmed)
```

The normal window stays below the gate and short-circuits. The anomalous window escalates, and the
causal stage recovers `shipping_latency`, matching the shipped ground truth.

## Notes

- **Gate calibration.** The threshold is set at runtime to the midpoint between the held-out normal
  and anomalous mean scores (the detector's operating point), so both branches are exercised: the
  healthy short-circuit and the escalate-to-explain path. The point of the example is the
  integration pattern, not detector accuracy. The held-out normal tail sits near the boundary, which
  is realistic for windows close to a regime change and is exactly why a calibrated gate matters.
- **Determinism.** Zero weight init plus a fixed data split make the printed numbers reproducible.
- **Extending it.** Swap the logistic regression for an MLP (add `candle-nn`), iterate over
  `carts_cpu_2` as well, or replace the midpoint gate with a learned threshold. None of these changes
  the detect, gate, explain shape.
