<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: A detector error model is a classical causal model over a `CausaloidGraph`

`DemModel` SHALL be a classical causal model in FStoch over detector and logical-observable
variables with error mechanisms as latent parents, carried on a `CausaloidGraph`, and SHALL be
constructible from any such graph without reference to a file format.

FStoch is a subcategory of QC (Lorenz & Tull, after Example 57), so a `DemModel` is a high-level
model an abstraction from a `CircuitModel` can target. `DemModel` is `qcm`-gated because the graph
type is.

#### Scenario: A model from a graph

- **WHEN** `DemModel::from_graph` is given a frozen `CausaloidGraph` with two error mechanisms, three
  detectors and one observable, each mechanism a parent of the detectors it flips
- **THEN** the model's variables are the detectors and the observable, its latents are the
  mechanisms, and `induced_dag()` has an edge from each mechanism to each detector it names

#### Scenario: An unfrozen graph is refused

- **WHEN** `DemModel::from_graph` is given a graph that is not frozen
- **THEN** it returns `QuantumError::CalculationError` stating that a frozen graph is required, as
  `FactorSupports::from_graph` does

### Requirement: The Stim text format is one constructor, behind a feature

Under the `dem` feature, `DemModel::from_stim_text` SHALL parse the `error(p) D… L…`,
`detector` and `logical_observable` lines of Stim's detector-error-model text format into a
`DemModel`, and SHALL refuse a line it does not recognise with `QuantumError::CalculationError`
naming the line. The `dem` feature SHALL imply `qcm` and SHALL add no external dependency.

#### Scenario: A three-line model parses

- **WHEN** the text `error(0.01) D0 D1\nerror(0.02) D1 L0\ndetector D0\n` is parsed
- **THEN** the model has two mechanisms with the stated probabilities, detectors `D0` and `D1`,
  observable `L0`, and the second mechanism is a parent of `D1` and `L0`

#### Scenario: An unknown directive is refused

- **WHEN** the text contains a line beginning `repeat 3 {`
- **THEN** parsing returns `CalculationError` naming that line, and no model is produced

### Requirement: The decoder is a black-box `τ` and is validated, never built

`DecoderAbstraction` SHALL be `Abstraction<CircuitModel, DemModel>` whose `τ` is the caller's
decoder as a channel from syndromes to logical outcomes, and the crate SHALL contain no `Decoder`
trait and no decoding algorithm.

#### Scenario: A decoder enters as a channel

- **WHEN** `DecoderAbstraction::new(circuit, dem, tau)` is given `tau` as a classical stochastic
  matrix from syndrome strings to logical outcomes
- **THEN** it is lifted to a `Channel` through the FStoch embedding, validated CPTP once, and the
  abstraction is built with no other information about how `tau` was computed

### Requirement: Naturality of the decoder abstraction is decoder-model validation

`check_naturality` on a `DecoderAbstraction` SHALL ask, for each opening in the low-level signature,
whether the decoder's classical picture commutes with the physical circuit, and a failing square
SHALL be reported with the physical query that exposed it as witness.

A failure is a fault the detector error model does not represent: a correlated error, a leakage
event or a hook error the model's mechanisms omit. The validation is causal, by openings, not
statistical.

#### Scenario: A correlated two-qubit error absent from the model is exposed

- **WHEN** the low-level circuit of a small memory experiment carries an injected two-qubit
  correlated error channel at a named location and the `DemModel` lists only single-qubit
  mechanisms
- **THEN** `check_naturality` rejects on the opening at that location, and the witness names the
  location

#### Scenario: A model that represents every mechanism passes

- **WHEN** the same circuit's `DemModel` includes the two-qubit mechanism with its probability
- **THEN** every square commutes within `Tolerance::state()` and the report accepts

### Requirement: Logical attribution ranks the low-level faults whose squares fail

Given a logical fault observed at the high level and a `FaultSet`, the attribution query SHALL
enumerate the low-level queries in the set whose naturality squares fail, SHALL rank them by
residual, and SHALL report the ranking with each entry's location and residual.

Because the low-level model is a circuit with its dilation, an entanglement-mediated correlation
between two detectors is represented as a shared quantum ancestor and not attributed to a classical
common cause.

#### Scenario: The injected location ranks first

- **WHEN** attribution runs on the memory experiment with one injected correlated error under
  `pauli_weight(1)` extended by the injected two-qubit fault
- **THEN** the first entry of the ranking names the injected location, and its residual exceeds
  every other entry's
