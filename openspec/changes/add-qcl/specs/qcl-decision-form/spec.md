<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: A decision is a record of measured, threshold, margin and verdict

QCL SHALL carry every decision in a `Check<R>` record holding the measured quantity, the threshold it
was compared against, the margin `measured / threshold`, the verdict, and an identifier for the item
examined, and SHALL aggregate the records of one decision in a `CheckReport<R>` exposing the worst
margin and the number of items examined.

The shape is taken from the shipped `CommutatorCheck<R>` in
`deep_causality_quantum/src/types/qcm/markov_freeze.rs`, whose fields are `node_j`, `node_k`, `norm`,
`threshold`, `margin` and `commutes`, and from `QuantumMarkovReport<R>`, which exposes
`worst_margin() -> Option<R>` and `tested_pairs() -> usize`. `Check<R>` and `CheckReport<R>`
generalise that pair over the item identifier, so a pair of graph nodes, an eigenvalue index, a code
generator and a hypothesis pair all fit one record. A margin at or below one accepts.

#### Scenario: A passing decision reports its distance from the edge

- **WHEN** a QCL stage accepts after comparing eleven items, the largest of their margins being 0.87
- **THEN** the returned `CheckReport<R>` yields `worst_margin()` of `Some(0.87)` and an examined
  count of eleven, so the caller reads how close the acceptance came to rejecting

#### Scenario: The zero-threshold convention follows the shipped check

- **WHEN** a `Check<R>` is recorded whose threshold is zero
- **THEN** the margin is the measured quantity itself when that quantity is positive and is zero when
  both are zero, which is the convention `quantum_markov_check` already applies in its
  `if threshold > R::zero()` branch, and the verdict rejects whenever the measured quantity is
  positive

### Requirement: No QCL stage returns a bare boolean

Every QCL stage that decides SHALL return a `CheckReport<R>` or a type containing one, and SHALL NOT
expose a decision whose only channel is a `bool` or a unit `Result<(), QuantumError>`.

A boolean is derivable from a report through `CheckReport::<R>::accepted()`. The derivation runs one
way, and the report is what crosses the stage boundary, because `worst_margin() == Some(0.87)` states
an acceptance and its remaining headroom while `true` states neither.

#### Scenario: Every stage signature carries the quantity it measured

- **WHEN** `check_markov`, `check_cptp`, `check_faithfulness`, `check_class_invariance`,
  `check_ldpc_weights`, `gate`, `design` or `adjudicate` is called through QCL
- **THEN** each returns a value from which the measured quantity, the threshold, the margin and the
  examined count are readable

#### Scenario: A rejection carries the record a pass carries

- **WHEN** a stage rejects an item
- **THEN** the report holds the rejecting item's measured quantity, threshold and margin, together
  with the count of items examined up to and including it, and the rejecting item is identifiable
  from the record

### Requirement: A check that examined nothing reports a vacuous pass

`CheckReport<R>` SHALL report the number of items examined, and SHALL label a report of zero examined
items as vacuous, distinguishable from an acceptance that examined at least one item.

`quantum_markov_check` skips factor pairs whose Hilbert supports are disjoint, because disjoint
supports impose no commutativity obligation. A factorization whose factors never overlap therefore
reaches `Ok` with `tested_pairs() == 0` and `worst_margin() == None`. QCL surfaces that state as a
vacuous pass, so the count obligation reaches the caller with the verdict.

#### Scenario: Pairwise disjoint supports certify nothing

- **WHEN** `check_markov` runs over a `ProcessFactors<R>` whose `FactorSupports` assign pairwise
  disjoint leg sets
- **THEN** the report has an examined count of zero, no worst margin, and a verdict that reads as
  vacuous, so a caller cannot record it as a certified commutation

#### Scenario: Vacuity survives a fold

- **WHEN** a vacuous report is folded into a pipeline verdict alongside reports that examined items
- **THEN** the folded report's examined count is the sum of the counts, and the fold reports no
  margin drawn from the vacuous member

### Requirement: Three shipped checks gain report-returning siblings

`check_completely_positive`, `check_trace_preserving` and `quantum_markov_check` SHALL each gain a
sibling returning a `CheckReport<R>`, and all three existing functions SHALL keep their current
names, signatures and behaviour.

The three compute the quantity the decision form needs and then discard it.
`check_completely_positive` and `check_trace_preserving` in
`deep_causality_quantum/src/types/qgates/channel.rs` return `Result<(), QuantumError>` after
computing a Hermiticity defect, a Choi spectrum and a trace defect. `quantum_markov_check` returns
`Err(QuantumError::CommutatorNonZero { .. })` on the first failing pair and drops the
`QuantumMarkovReport<R>` it had been filling, so a rejected candidate reports no margins and no
`tested_pairs()`. The siblings are named `check_completely_positive_report`,
`check_trace_preserving_report` and `quantum_markov_check_report`, and each SHALL delegate to the
shipped computation rather than restate it.

#### Scenario: The CP defect and the spectrum reach the caller

- **WHEN** `check_completely_positive_report(choi, tol)` runs on a Choi operator whose smallest
  eigenvalue is negative
- **THEN** the report carries the Hermiticity defect and the minimum eigenvalue, each against `tol`,
  and reports the number of eigenvalues examined, while `check_completely_positive(choi, tol)` on the
  same input still returns `Err(QuantumError::NonCptpChannel(..))` with its signature unchanged

#### Scenario: The TP defect reaches the caller

- **WHEN** `check_trace_preserving_report(choi, d_in, d_out, tol)` runs
- **THEN** the report carries `max |Tr_out(J) − I_in|` as the measured quantity, `tol` as the
  threshold, and the number of compared entries of the `d_in × d_in` residual as the examined count,
  on the accepting path as well as the rejecting one

#### Scenario: The Markov failure path keeps its report

- **WHEN** `quantum_markov_check_report` reaches a factor pair whose commutator norm exceeds the
  Q-TOL threshold
- **THEN** it returns a report carrying every pair tested up to and including the rejecting one, with
  that pair's norm, threshold and margin, and reserves `Err` for structural failures such as a
  `FactorSupports::validate` rejection or a shape mismatch

### Requirement: The tolerance family derives every member from the scalar

`Tolerance<R>` SHALL be a family of named members over `R: RealField`, each a function of
`R::epsilon()`, covering the four policies the crate ships today and delegating to their
implementations.

The four are the Q-TOL commutator policy of `CommutatorTolerance<R>`, whose threshold is
`C·(‖ρ_j‖·b_k + ‖ρ_k‖·b_j + 2·γ_n·‖ρ_j‖·‖ρ_k‖)`; the validation policy `√ε` of
`Projection::<R, D>::default_tolerance`; the numerical-rank cutoff `D·ε·scale` of
`Projection::range_projector`; and the state-validation policy of `DensityMatrix::default_tolerance`,
overridable through `DensityMatrix::with_tolerance`. Each member keeps the shape its own check needs,
and the family names them so a caller selects a policy instead of writing a literal.

#### Scenario: Widening the scalar tightens every member

- **WHEN** `FloatType` is changed from `f64` to a wider real carrier
- **THEN** every member of `Tolerance<R>` tightens with `R::epsilon()` and no call site changes,
  because no member holds a numeric literal as its threshold

#### Scenario: The numerical-rank member stays a rank cutoff

- **WHEN** the numerical-rank member is requested for a `D`-dimensional operator
- **THEN** it yields `D·ε·scale` rather than the `√ε` validation value, preserving the property
  `range(P) + range(Q) = range(P + Q)` that `Projection::range_projector` relies on

#### Scenario: An exact quantity takes no member at all

- **WHEN** a check measures a quantity that is exact by construction, such as the rational phase
  residual of a `DiagonalPhase<W>` gate under `is_diagonal_trivial`
- **THEN** it reports that residual against a threshold of exactly zero, together with a count of the
  code-space elements examined, and requests no `Tolerance<R>` member

### Requirement: The tolerance family has no integer member

`Tolerance<R>` SHALL be parameterised over the real carrier alone, and SHALL expose no member for a
count, a shot budget, or any other quantity carried on the integer axis.

Widening `FloatType` buys accuracy whose error is bounded by `epsilon()`, which is what every
tolerance derives from. Widening `IntType` buys headroom against overflow, which nothing bounds. A
count is right or overflowed, and overflow is a hard wrongness rather than a graded one. The integer
axis therefore gets checked arithmetic: `NaturalNumber::checked_difference` returns `None` when the
difference does not exist in ℕ, and `monus` clamps to zero, which is exactly a ledger draw-down and
needs no hand-written guard.

#### Scenario: No constructor admits a count

- **WHEN** a caller attempts to build a `Tolerance` over a type implementing `NaturalNumber`
- **THEN** the program fails to compile, because the family's bound is `R: RealField` and no member
  is defined over the integer axis

#### Scenario: An overdrawn budget is reported by the arithmetic

- **WHEN** a ledger draw-down subtracts a shot count larger than the remaining budget
- **THEN** `checked_difference` returns `None` and the stage reports the overdraw as a structured
  failure, with no threshold, no margin and no tolerance involved in the decision
