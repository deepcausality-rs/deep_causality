<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Qubit operators are built by name and own their shape

`QubitOperator<R>` SHALL construct every single-qubit operator a QCL stage uses through a named
constructor, covering `identity`, `pauli_x`, `pauli_y`, `pauli_z`, `hadamard`, `rotation(axis, angle)`
and `phase(theta)`, and SHALL own the shape of its `CausalTensor<Complex<R>>` interior so that a
caller never assembles a flat complex slice against a hand-computed row stride.

That alphabet is the one the crate already names. `qgates/gates.rs` declares
`trait QuantumGates { gate_identity, gate_x, gate_y, gate_z, gate_hadamard, gate_cnot }` and no type
in the workspace implements it, so today a program that wants Pauli-X writes the four `Complex`
entries itself and calls `CausalTensor::from_slice(&data, &[2, 2])`. The carrier SHALL be generic in
the scalar under the `R: RealField` bound the shipped operator layer uses, and SHALL expose its
interior as `&CausalTensor<Complex<R>>` so that `matrix_commutator`, `embed_on_legs`,
`hermiticity_defect` and `choi_from_kraus` accept it with no conversion.

#### Scenario: A named constructor replaces a packed slice

- **WHEN** a program needs Pauli-X
- **THEN** `QubitOperator::pauli_x()` returns a carrier reporting `dim() == 2`, whose
  `matrix()` is the `[2, 2]` tensor that the shipped channel and linalg functions consume directly

#### Scenario: The named constructors are unitary by construction

- **WHEN** any of `pauli_x`, `pauli_y`, `pauli_z`, `hadamard`, `rotation(axis, angle)` or
  `phase(theta)` returns an operator `U`
- **THEN** `U·U†` equals the identity within `R::epsilon().sqrt()`, so `Channel::unitary` admits it
  without a further test

#### Scenario: A non-finite parameter is rejected at construction

- **WHEN** `rotation(axis, angle)` or `phase(theta)` is called with a non-finite argument
- **THEN** construction returns `QuantumError::NonFiniteValue` and no operator value exists

### Requirement: A Channel is CPTP-checked once, at construction

`Channel<R>` SHALL validate complete positivity and trace preservation once, at construction, and
SHALL NOT re-check either property on any later application. Construction builds the Choi operator
with the shipped `choi_from_kraus` and runs the shipped `check_completely_positive` and
`check_trace_preserving` against a tolerance supplied by the `Tolerance<R>` family, which is finite
and non-negative and therefore passes the shipped `validate_tolerance` guard.

A failed construction SHALL return the structured error the shipped check produced, which is
`QuantumError::NonCptpChannel` for a negative Choi eigenvalue or a `Tr_out(J) ≠ I_in` defect and
`QuantumError::NonPositiveOperator` for a non-Hermitian Choi, so no partially validated channel value
is reachable.

Unitary evolution SHALL enter through a named conversion, `Channel::unitary(&QubitOperator<R>)`,
which builds the one-element Kraus family itself. This closes the friction where a unitary was passed
to `apply_kraus(&[u], rho)` and read as a channel at the call site.

#### Scenario: A non-trace-preserving family fails at construction

- **WHEN** a Kraus family whose `Tr_out(J)` differs from `I_in` is offered to `Channel::from_kraus`
- **THEN** construction returns `QuantumError::NonCptpChannel` carrying the measured
  `max |Tr_out(J) − I|` defect, and no `Channel` value is produced

#### Scenario: Application does not re-validate

- **WHEN** a constructed `Channel` is applied to one thousand states in a sweep
- **THEN** `check_completely_positive` and `check_trace_preserving` each ran exactly once, and every
  application routes to `apply_kraus` or `apply_choi` alone

#### Scenario: Composition inherits CPTP rather than re-deriving it

- **WHEN** two channels are composed over a shared wire
- **THEN** the result is a `Channel` built by `choi_compose` with no CPTP re-validation, because
  complete positivity and trace preservation are properties of the composed maps and are inherited by
  the composite, which is what `choi_compose`'s `RealField`-only bound already records

### Requirement: QuantumPlant seals a validated state and evolves by operation

`QuantumPlant<R>` SHALL seal a validated `DensityMatrix<R>` and SHALL expose evolution as an
operation returning a new plant, with no accessor yielding `&mut` to the state.
`evolve(&self, channel: &Channel<R>)` applies the channel through the channel's own validated Kraus
family and validates the result once as a `DensityMatrix<R>`, so every state a caller can observe has
passed the Hermiticity, positivity and unit-trace checks in `DensityMatrix::with_tolerance`.

#### Scenario: Evolution yields a new sealed plant

- **WHEN** a plant is evolved by a channel
- **THEN** a new `QuantumPlant<R>` carries the evolved state, the receiver compares equal to its
  pre-evolution value, and the evolved state satisfies `|Tr(ρ) − 1| ≤ tol` and has no eigenvalue below
  `−tol`

#### Scenario: A dimension disagreement is caught before a state is built

- **WHEN** a channel whose input dimension differs from the plant's dimension is applied
- **THEN** `evolve` returns `QuantumError::DimensionMismatch` from the shipped `apply_kraus` shape
  check, and no `DensityMatrix` is constructed

### Requirement: An Observable is a named projector carrying its own read-out

`Observable<R, D>` SHALL pair a name with a validated `Projection<R, D>` and SHALL carry its own
read-out, returning the Born probability `Tr(Pρ)` through the shipped `born_projective_probability`
and the `Prob` verdict through `born_projective_prob`. The read-out is the measurement boundary of
the verdict law: an `Observable` SHALL be the only site at which a verdict enters a QCL pipeline, and
every stage upstream of it carries operators.

An `Observable` SHALL expose its `Projection<R, D>` so that `adjudicate` can apply
`Projection::commutes_with` to a pair of projection-valued verdicts, and SHALL fold no verdicts
itself, because the fold rule depends on which kind of verdict a world carries.

#### Scenario: A ket becomes a named read-out in one step

- **WHEN** `Observable::from_ket("excited_population", &ket)` is constructed
- **THEN** the projector is validated by the shipped `Projection::from_ket` as a rank-1 Hermitian
  idempotent, and the observable reads out a probability in `[0, 1]` against a plant of dimension `D`

#### Scenario: A dimension mismatch is reported by the shipped boundary

- **WHEN** an observable of dimension `D` reads out against a plant whose `dim()` differs from `D`
- **THEN** the read-out returns `QuantumError::DimensionMismatch` naming both dimensions, which is
  the error `born_projective_probability` already raises

#### Scenario: Non-commuting projectors stay visible to the fold

- **WHEN** two observables are built from projections in general position
- **THEN** `a.projection().commutes_with(b.projection())` reports `false`, and the observables carry
  their read-outs individually for `adjudicate` to fold under the verdict law

### Requirement: Every carrier seals its interior and exposes operations

Every QCL carrier SHALL expose read accessors and operations only, and SHALL NOT expose any method
yielding `&mut` to a field whose invariant was established at construction. `EnvironmentalPrep`
already realises the rule: it wraps a validated `DensityMatrix<R>` and exposes `state()`, `matrix()`
and `dim()`, so a model threading `ρ_A` through evaluation cannot alter the preparation mid-pass and
its result is reproducible.

`QubitOperator`, `Channel`, `QuantumPlant` and `Observable` SHALL follow it. A change to a carrier's
contents is expressed as an operation returning a new carrier, which re-runs the construction checks
that the new contents require.

#### Scenario: A changed channel is a second construction

- **WHEN** a caller holding a `Channel<R>` wants a different Kraus family
- **THEN** it calls `Channel::from_kraus` again, the CPTP checks run against the new family, and the
  first channel is unaffected

#### Scenario: Two stages reading one plant see one state

- **WHEN** the same `QuantumPlant<R>` is passed by reference to two independent stages
- **THEN** both read the identical state, and the plant compares equal to its pre-stage value after
  both have run

### Requirement: Every fallible carrier operation has a wrappers.rs-style lift

Every carrier operation that can fail SHALL have a lift into the causal monad in the shape
`qgates/wrappers.rs` already uses: a free function returning `PropagatingEffect<T>` that maps `Ok(v)`
to `PropagatingEffect::pure(v)` and `Err(e)` to `PropagatingEffect::from_error(CausalityError::from(e))`.

The eleven functions in `wrappers.rs` are the pattern and the new lifts sit beside them:
`born_probability`, `expectation_value`, `apply_gate`, `commutator`, `fidelity`, and the six Haruna
gate builders `haruna_z_gate`, `haruna_x_gate`, `haruna_s_gate`, `haruna_t_gate`, `haruna_cz_gate`
and `haruna_hadamard_gate`. QCL SHALL introduce no effect type of its own; the categorical structure
it threads is `PropagatingEffect`.

#### Scenario: A stage reads a value rather than a Result

- **WHEN** a plant is evolved inside a causal flow
- **THEN** the lift returns `PropagatingEffect<QuantumPlant<R>>` and the caller writes no match arm,
  no turbofish and no closure, which is the ceremony the pattern hides at the kernel boundary

#### Scenario: A failure short-circuits with its structured cause

- **WHEN** a lifted construction fails, such as a non-CPTP Kraus family offered to a lifted
  `Channel::from_kraus`
- **THEN** the returned effect carries the `CausalityError` converted from the structured
  `QuantumError`, downstream stages short-circuit, and the cause survives as a typed error rather
  than a display string

### Requirement: Every carrier is generic over its scalar and names no width of its own

Every carrier SHALL be generic over the scalar it carries and SHALL NOT name a concrete float or
integer width internally, so that a program fixes one alias at `QclBuilder::config` and the whole
pipeline instantiates at that precision.

The bound each carrier declares SHALL be the weakest structure that carries its operations, with one
exception stated rather than hidden: a carrier holding `Complex<R>` is pinned at `R: RealField` by
the complex tower itself, since every impl for `Complex<T>` in `deep_causality_num_complex` is
written `impl<T: RealField>`. A carrier so pinned SHALL say that the bound comes from the carrier
rather than from its operations, so a reader does not look for a relaxation that is not there.

#### Scenario: A carrier is lifted between precisions as an operation

- **WHEN** a validated operator carried as `CausalTensor<Complex<R>>` is lifted from one scalar to
  another
- **THEN** the lift is a composition of two functors, the outer over cells and the inner over the
  real and imaginary slots, and the carrier's invariant survives it

#### Scenario: A carrier's tolerance moves with its scalar

- **WHEN** a `Channel` or a `QuantumPlant` validates its interior at construction
- **THEN** the tolerance it validates against is drawn from the `Tolerance<R>` family and is a
  function of `R::epsilon()`, so widening the scalar tightens the check with no code change
