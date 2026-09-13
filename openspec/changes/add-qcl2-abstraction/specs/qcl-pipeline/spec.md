<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: The builder accepts a circuit model as a fourth subject

`QclBuilder::config::<R, N>().over_circuit(model)` SHALL produce a `CircuitSubject` beside the
plant, model and code subjects, `build()` SHALL refuse a circuit whose induced DAG has a directed
cycle with `CyclicStructureUnsupported`, and `.over_model(graph, factors, supports)` SHALL remain
for callers who hold only the marginal.

A `Screened<R>` produced from a circuit subject records that its factorization came from a
dilation, so a later abstraction constructor can tell it from one built over a bare process
operator. The constructor is `qcm`-gated with the dilation it runs.

#### Scenario: A circuit builds and its dilation is screened

- **WHEN** a two-node unitary circuit is passed to `.over_circuit` and `validate` runs
  `check_markov` on it
- **THEN** the stage runs on the dilation's factors, the report accepts within Q-TOL, and the
  `Screened<R>` reports its origin as a circuit

#### Scenario: A cyclic circuit is refused at build

- **WHEN** a circuit's grouping wires node `A` into node `B` and node `B` back into node `A`
- **THEN** `build()` returns `CyclicStructureUnsupported` before any stage runs

### Requirement: `validate` gains the abstraction stages

On a circuit subject paired with an `Abstraction`, `validate` SHALL offer
`check_alignment_structure` before any operator is formed and `check_naturality` after it, each
recorded as a named stage in `Screened::stages()` with its `CheckReport<R>`, and the first failure
SHALL be sticky as it is for every other stage.

#### Scenario: The precheck runs before the operator check

- **WHEN** `validate(&cfg).check_alignment_structure(&abstraction).check_naturality(&abstraction)`
  runs and the precheck rejects
- **THEN** `check_naturality` does not run, no matrix is formed, and `finalize` carries the
  precheck's structured error out

#### Scenario: Both stages record

- **WHEN** both stages accept
- **THEN** `stages()` lists `("check_alignment_structure", …)` then `("check_naturality", …)` and
  the folded report's examined count is their sum

### Requirement: The crosstalk consumer reproduces its decision over circuit-derived candidates

The pipeline SHALL admit the same three crosstalk candidates by Markov and C₃, SHALL refuse the
cyclic fourth at `build()`, SHALL plan `{do(Q1), do(Q2)}` at cost 2 against tomography at 200, and
SHALL name H₁ the survivor, when each admitted candidate is re-expressed as a `CircuitModel` whose
dilation yields normalised factors with the same parental structure.

The v1 consumer's factors are legal for the commutation check and are not Choi operators of any
channel, so no dilation reproduces their values; the decision is what is reproduced.

#### Scenario: The screen and the plan are unchanged

- **WHEN** the crosstalk example runs `.over_circuit` for each candidate and the same probes and
  baseline as the v1 example
- **THEN** three candidates are admitted, the plan's entries are `do(Q1)` and `do(Q2)` at total
  cost 2, and the adjudication names `H1 Q1->Q2` the survivor

### Requirement: The live `qcl-*` specifications are restored before implementation

The seven live specifications SHALL be restored from the archived `add-qcl` deltas, text for text,
before any implementation task of this change runs: `qcl-carriers`, `qcl-code-checks`,
`qcl-decision-form`, `qcl-evidence`, `qcl-experiment-design`, `qcl-hypothesis` and `qcl-pipeline`,
and `openspec validate --specs` SHALL pass on them.

The archive step of `add-qcl` did not merge its deltas, so the seven directories exist and are
empty. This change writes every requirement against an existing capability as `ADDED` so it
validates whether or not the restore has run, and the restore is its first task.

#### Scenario: The restored specifications validate

- **WHEN** the seven live `spec.md` files are written from the archived deltas
- **THEN** each carries the archived requirement count (7, 11, 7, 7, 7, 9 and 9) and
  `openspec validate --specs` reports no error
