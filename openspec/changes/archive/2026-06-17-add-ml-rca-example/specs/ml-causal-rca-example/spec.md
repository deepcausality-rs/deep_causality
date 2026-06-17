## ADDED Requirements

### Requirement: Labeled dataset loading

The example SHALL load the shipped Sock Shop dataset from
`examples/causal_discovery_examples/data/sock-shop-2/<case>/` and construct a labeled
feature matrix in which every row of `normal.csv` is labeled `0` and every row of
`anomalous.csv` is labeled `1`. The 44 numeric metric columns SHALL be used as features and the
column order SHALL be preserved so that a feature index maps back to a named metric. The example
MUST NOT add or ship any new data file.

#### Scenario: Both label classes loaded from the shipped CSVs

- **WHEN** the example loads the chosen Sock Shop case
- **THEN** it reads `normal.csv` and `anomalous.csv` from the existing data directory
- **AND** produces a feature matrix of 44 columns with a label vector containing both `0` (normal) and `1` (anomalous) rows

#### Scenario: Missing data is reported, not panicked past silently

- **WHEN** a required CSV file is absent or unreadable
- **THEN** the example terminates with a clear error message naming the missing path
- **AND** does not proceed to training on partial data

### Requirement: Candle anomaly classifier (detect stage)

The example SHALL train a Candle model (logistic regression or a one-hidden-layer MLP) on the
labeled feature matrix and SHALL emit a scalar anomaly score in `[0, 1]` for an input window. The
model, training loop, and inference SHALL use the Candle framework. Candle SHALL be a dependency of
the `causal_discovery_examples` example crate only; no `deep_causality*` library crate may gain an
ML dependency.

#### Scenario: Trained classifier separates the two classes

- **WHEN** the classifier is trained on the labeled matrix and then scored on held-back normal and anomalous rows
- **THEN** anomalous rows receive a higher mean anomaly score than normal rows
- **AND** the model produces scores within `[0, 1]`

#### Scenario: ML dependency confined to the example

- **WHEN** the workspace is inspected
- **THEN** `candle-*` appears only under the `causal_discovery_examples` example crate's manifest
- **AND** no `deep_causality_*` library crate declares a Candle dependency

### Requirement: Anomaly gate

The example SHALL apply a documented threshold to the Candle anomaly score to decide whether to
escalate to the causal explanation stage. When the score is below the threshold the example SHALL
report "healthy / no escalation" and SHALL NOT run the causal root-cause ranking; when the score is
at or above the threshold the example SHALL escalate.

#### Scenario: Anomalous window escalates

- **WHEN** the anomaly score for an anomalous window is at or above the threshold
- **THEN** the example escalates to the causal explanation stage

#### Scenario: Normal window does not escalate

- **WHEN** the anomaly score for a normal window is below the threshold
- **THEN** the example reports a healthy verdict and skips the causal stage

### Requirement: Causal root-cause explanation (explain stage)

When escalation occurs, the example SHALL invoke the existing `deep_causality` causal
root-cause ranking (the BRCD/SURD path already used by `example_brcd_discovery`) over the anomalous
data and SHALL produce a ranked list of candidate culprit metrics. The example SHALL reuse the
existing causal-discovery machinery without modifying the causal-discovery algorithms.

#### Scenario: Escalation produces a ranked culprit list

- **WHEN** the causal stage runs on the anomalous window
- **THEN** the example outputs a ranked list of candidate root-cause metrics (best first)

### Requirement: Monadic bridge between the two stages

The example SHALL sequence the detect and explain stages through a `PropagatingProcess` so that the
Candle gate result determines whether the causal stage runs, the carried value advances to the
root-cause verdict on escalation, and the escalation decision is recorded in the process
`EffectLog`. A below-threshold (no-escalation) run SHALL short-circuit the chain without producing a
root-cause verdict.

#### Scenario: Escalation is recorded in the effect log

- **WHEN** an anomalous window drives the chain to escalate
- **THEN** the `EffectLog` of the resulting process records the escalation
- **AND** the final carried value is the root-cause verdict

#### Scenario: No-escalation short-circuits the chain

- **WHEN** a normal window keeps the score below threshold
- **THEN** the chain short-circuits without a root-cause verdict in the carried value

### Requirement: Verdict compared against shipped ground truth

The example SHALL compare the top-ranked culprit from the causal stage against the shipped ground
truth (`expected.txt` / `notes.txt`) for the chosen case and SHALL print whether the top-ranked
metric matches (or appears within the top-k of) the ground-truth ranking. The comparison output
SHALL make the "ML detected, causality explained, ground truth confirmed" narrative explicit.

#### Scenario: Verdict checked against ground truth

- **WHEN** the causal stage yields its ranked culprit list
- **THEN** the example reads the shipped ground-truth ranking for the case
- **AND** reports whether the top-ranked culprit matches the ground-truth top cause (or is within the reported top-k)

### Requirement: Runnable, documented example

The example SHALL be runnable via
`cargo run -p causal_discovery_examples --example example_ml_rca`, SHALL carry module-level (`//!`)
documentation, and SHALL ship a `README.md` following the standard example template. The example
SHALL be registered in the `causal_discovery_examples` `Cargo.toml`, the
`causal_discovery_examples/README.md` table, and the top-level `examples/README.md` table.

#### Scenario: Example runs end to end

- **WHEN** `cargo run -p causal_discovery_examples --example example_ml_rca` is executed
- **THEN** the example trains the classifier, gates, runs the causal stage on an anomalous window, and prints the ground-truth comparison without error

#### Scenario: Example is discoverable in the docs

- **WHEN** the example READMEs are inspected
- **THEN** `example_ml_rca` appears in the `causal_discovery_examples` table and the top-level examples overview with its run command
