## Why

The repository has no example that integrates a mainstream Rust ML framework with `deep_causality`. The two are complementary on a problem they each solve only halfway: a supervised classifier can cheaply **detect** that a distributed system is anomalous but cannot say *which* component is at fault, while `deep_causality`'s causal-discovery pipeline **explains** the root cause but has no learned anomaly detector to gate it. The shipped, labeled Sock Shop microservice dataset (`examples/causal_discovery_examples/data/sock-shop-2/`) already powers the BRCD root-cause example and is a ready-made supervised training set, so the integration can be shown end to end with zero new data and no changes to any library crate.

## What Changes

- Add a new runnable example `example_ml_rca` to the **`causal_discovery_examples`** crate demonstrating the **"ML detects, causality explains"** pattern by combining **Candle** (the HuggingFace Rust ML framework) with `deep_causality`.
- **Detect stage (Candle):** train a small classifier (logistic regression / one-hidden-layer MLP) on the labeled Sock Shop CSVs (`normal.csv` label 0, `anomalous.csv` label 1; 44 numeric features, ~720 rows) and emit a per-window **anomaly score**.
- **Explain stage (`deep_causality`):** when the anomaly score crosses a threshold, run the **existing** BRCD/SURD causal root-cause ranking over the anomalous window to rank the culprit microservice/metric, and compare the top-ranked result against the shipped ground truth (`expected.txt` / `notes.txt`).
- **Bridge:** sequence the two stages through a `PropagatingProcess` monad — the Candle gate decides whether to fire the causal stage; the carried value becomes the root-cause verdict; the `EffectLog` records the escalation.
- Add **Candle as an example-only dependency** of `causal_discovery_examples` (dev/example scope). No library crate gains an ML dependency.
- Add a `README.md` for the example following the standard example template, and register the example in the crate's `Cargo.toml` and the top-level `examples/README.md` + `causal_discovery_examples/README.md` tables.

## Capabilities

### New Capabilities
- `ml-causal-rca-example`: a runnable, toy-scale example that trains a Candle anomaly classifier on the shipped Sock Shop dataset and feeds its gated output into the existing `deep_causality` causal root-cause ranking through a `PropagatingProcess`, demonstrating the Candle ↔ `deep_causality` integration pattern (ML detection escalating to causal explanation) with a verdict checked against shipped ground truth.

### Modified Capabilities
<!-- None. No existing spec in openspec/specs/ changes its requirements. This is an
     examples-only addition; the BRCD / causal-discovery library behavior is reused unchanged. -->

## Impact

- **New code:** one example binary under `examples/causal_discovery_examples/` (Candle data-loading + training + scoring, the gate, and the call into the existing causal root-cause pipeline) plus its `README.md`.
- **Dependencies:** `candle-core` (and likely `candle-nn`) added under the `causal_discovery_examples` example crate only. Examples are exempt from the repo's no-external-runtime-dependency rule; library crates (`deep_causality*`) remain ML-free and unchanged.
- **No behavior change** to any library crate or existing example. The Sock Shop dataset, the CSV ingestion, and the BRCD/SURD root-cause machinery are reused as-is.
- **Docs:** new example row in `examples/README.md` and `examples/causal_discovery_examples/README.md`.
- **Out of scope:** production-grade model accuracy, GPU/feature-flagged Candle backends, streaming/online training, and any change to the causal-discovery algorithms themselves.
