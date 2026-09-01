<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: `derive_code` reads the code parameters off the chain complex

`derive_code` SHALL derive a CSS code from the configured chain complex, taking `n` from the number
of 1-cells, `k` from the Betti number `β₁` over 𝔽₂, the Z-stabilizer generators from the columns of
`∂₂` and the X-stabilizer generators from the columns of `δ₀`.

All three sources ship. `ChainComplex::num_cells(1)` counts the qubits;
`homology_representatives::<W>(1)` returns the `β₁` logical `Z` generators that
`LogicalBasis::from_complex` already stores and `num_logical_qubits()` already reports, and
`betti_number_over(1, HomologyField::Gf2)` is the same count from the ranks; `boundary_matrix(2)` and
`coboundary_matrix(0)` carry the checks, reduced mod two by `csr_to_packed_gf2_mod2::<W>` and read
column by column with `Gf2Chain::from_column`. `derive_code` composes them and adds no linear
algebra of its own.

#### Scenario: The lattice subject derives [[32, 2]]

- **WHEN** `derive_code` runs on `LatticeComplex::<2, FloatType>::square_torus(4)`
- **THEN** it reports `n = 32` from the 32 1-cells, `k = 2` from `β₁` over 𝔽₂, 16 Z-stabilizer
  generators from the 16 `∂₂` columns and 16 X-stabilizer generators from the 16 `δ₀` columns

#### Scenario: A simplicial subject derives its own numbers

- **WHEN** `derive_code` runs on the 3×3 simplicial torus `torus_2` from
  `deep_causality_homology::utils_tests::reference_spaces`
- **THEN** it reports `n = 27`, `k = 2`, 18 Z-stabilizer generators of weight 3 and 9 X-stabilizer
  generators of weight 6, so the parameters follow the complex it was handed

#### Scenario: No distance is claimed

- **WHEN** a caller reads the value `derive_code` returned
- **THEN** it carries `n`, `k` and the two stabilizer families and exposes no `d`, because the
  minimum weight of a non-trivial class is a computation this change does not add

### Requirement: `check_ldpc_weights` decides both weights against a declared bound

`check_ldpc_weights` SHALL report the maximum column weight and the maximum row weight of both check
matrices against a bound the configuration declares, and SHALL report how many rows and columns it
examined.

A column of `∂₂` or `δ₀` is one stabilizer generator, so its weight is the number of qubits that
check acts on. A row is one qubit, so its weight is the number of checks that act on it. LDPC asks
for both to stay bounded as the code grows, so both are measured and one declared bound decides them,
in the `(measured, threshold, margin, verdict)` shape `qcl-decision-form` fixes.

#### Scenario: The lattice subject passes at a bound of four

- **WHEN** `check_ldpc_weights` runs with a declared bound of 4 on the code derived from
  `LatticeComplex::<2, FloatType>::square_torus(4)`
- **THEN** it accepts, reporting a maximum column weight of 4 because each face meets four edges, a
  maximum row weight of 2 because each edge lies in two faces and meets two vertices, and 96 items
  examined, being the 32 rows and 16 columns of each of the two check matrices

#### Scenario: A bound of three rejects and names the offender

- **WHEN** the same code is checked against a declared bound of 3
- **THEN** it rejects with a margin of `4 / 3`, names the check matrix and the column index whose
  weight is 4, and reports the count of items examined up to and including that column

#### Scenario: A complex with no 2-cells reads as a vacuous pass

- **WHEN** `check_ldpc_weights` runs on a complex whose `num_cells(2)` is zero
- **THEN** the Z half reports zero columns examined, so a code carrying no Z-stabilizer is visible as
  an empty check

### Requirement: `check_class_invariance` is decided over the code space

`check_class_invariance` SHALL decide whether the ratio `U(γ ⊕ b) U(γ)†` is the identity on the code
space, and SHALL NOT decide whether that ratio commutes with the logical `X` generators over the full
Hilbert space. The cohomology basis is not consulted.

Haruna's Eq. (3.20) decomposes `S(γ₁) = S(γ₂) · S(∂₂f) · exp(iπ a(γ₂) a(∂₂f))`, and the paper's words
for the last factor are that it "behaves trivially on the code space". It is a *controlled*
stabilizer: it is not trivial on the whole Hilbert space and does not commute with every `X̄` there.
So the defect in the shipped criterion is not that its quantifier is too wide but that it asks a
different question, and widening or narrowing the state set cannot repair it.

A diagonal ratio has phase `Q(|(γ ⊕ b) ∩ x|)/M − Q(|γ ∩ x|)/M`, so it is the identity on the code
space exactly when that phase is a whole number of turns for every basis state the code space
contains. Those are the states meeting every Z-stabilizer evenly. The X-stabilizers SHALL NOT be
carried, because they fix which superpositions are code states and a diagonal phase does not see a
superposition.

Of the Z-stabilizer constraints, the enumeration SHALL impose the one the shift itself carries,
`|b ∩ x|` even, and SHALL impose it only when `b` lies in the stabilizer span. That is the factor
Eq. (3.20) turns on. Dropping the others is a **sound relaxation** — it can only enlarge the state
set, so a pass here is a pass on the code space — and it is what keeps the predicate discriminating,
since a shift by a non-boundary then imposes nothing.

#### Scenario: The full-space criterion rejects sound gates, as measured

- **WHEN** the commutation criterion runs on the 3×3 simplicial torus over its 18 `∂₂` boundaries and
  its 2 cohomology generators
- **THEN** `Z̄` reports `holds == true` on all 36 (boundary, cohomology generator) pairs, while `S̄`
  and `T̄` report `holds == false` with the failure on the first boundary, which is the wrong answer
  for two of the three gates Table 1 certifies

#### Scenario: The code-space criterion admits S̄ and T̄ on the same subject

- **WHEN** the same three gates are checked against a `LogicalBasis` carrying the torus's stabilizer
  generators
- **THEN** `Z̄`, `S̄` and `T̄` each report `holds == true` with `tested == 18`, one per boundary, and
  `first_failure == None`

#### Scenario: The restriction is load-bearing in both directions

- **WHEN** the code-space restriction is dropped, and separately when it is applied unconditionally
  to every shift
- **THEN** the first makes `S̄` and `T̄` fail on a genuine boundary and the second makes a shift by a
  non-boundary pass, so neither the restriction nor its guard can be removed silently

#### Scenario: The relaxation still rejects a change of class

- **WHEN** `Z̄(γ₀)` is checked against a shift by `γ₁`, the second homology generator, which is no
  boundary
- **THEN** the report is `holds == false` with `tested == 1`, because `γ₁` is outside the stabilizer
  span, no parity constraint is imposed, and the odd states that witness the change of class remain
  in the enumeration

#### Scenario: The enumeration is bounded by weight, not by register size

- **WHEN** a shift is checked
- **THEN** the states visited are the occupancies of the three blocks `γ ∩ b`, `b \ γ` and `γ \ b`,
  which determine both overlap counts and so determine the phase, and the count is bounded by the two
  chains' weights rather than by `2ⁿ`

### Requirement: `LogicalBasis` derives the stabilizer generators it needs

`LogicalBasis<W>::from_complex` SHALL derive the Z-stabilizer generators as a basis of `im ∂ₖ₊₁` and
carry them, and SHALL expose them through an accessor. There SHALL NOT be a second constructor, and
there SHALL NOT be an error path for a basis carrying no stabilizers.

`from_complex` already holds the chain complex, which is the only thing a stabilizer basis needs, so
requiring a caller to supply the generators separately would add a way to build a basis that answers
the class-invariance question wrongly. Deriving them removes that state rather than validating it.

The generators SHALL be an image basis rather than the raw `∂ₖ₊₁` columns. The columns carry one
dependency per `(k+1)`-cycle, and a dependent generating set inflates the span test's cost without
changing its answer.

`is_logically_trivial` and `are_logically_equivalent` are untouched: B.1's commutation criterion over
Paulis needs no stabilizer group, which is why the shipped doc block records that its precondition
goes unchecked.

#### Scenario: The generators are an independent basis of the boundaries

- **WHEN** the 3×3 simplicial torus is read at grade 1, where `β₂(T²) = 1` over 𝔽₂ gives exactly one
  dependency among the `∂₂` columns
- **THEN** the stabilizer count is one fewer than the column count, and the rank equals the count

#### Scenario: Every stabilizer induces a trivial gate

- **WHEN** each stabilizer generator is read as `Z̄`, `S̄` and `T̄`
- **THEN** `is_diagonal_trivial` accepts all three, which is Haruna Eq. (3.21) for the whole family
  rather than for `S̄` alone

#### Scenario: The Pauli path is unchanged

- **WHEN** a `LogicalBasis` decides `is_logically_trivial` for the `Z̄` Paulis over the torus's
  homology generators and its `∂₂` columns
- **THEN** the answers match `is_diagonal_trivial` on the corresponding `DiagonalPhase::z` gates,
  which is the agreement `test_the_diagonal_predicate_agrees_with_the_pauli_one_on_z` already asserts

### Requirement: The phase arithmetic is exact, so the check carries no tolerance

Every diagonal gate phase SHALL be carried as `Turns = Rational<i64>`, evaluated as `Q(n) / M` for an
integer polynomial `Q` and a power-of-two `M`, and every acceptance test SHALL ask whether a phase
difference is integral.

`DiagonalPhase::phase_at` returns `Rational::new(acc, self.modulus())` with no float anywhere on the
path, and the three shipped polynomials are `Q(n) = n` at `M = 2`, `Q(n) = n²` at `M = 4` and
`Q(n) = 2n³ − 3n² + 2n` at `M = 8`. A phase lives in ℝ/ℤ, so `is_integer()` on a difference is the
question and the residual is exactly zero or exactly not. `check_class_invariance` reports that exact
residual with its count, where §3.1's table wrote a tolerance.

#### Scenario: The polynomials reproduce their single-qubit gates

- **WHEN** `phase_at(1)` is read on `DiagonalPhase::z`, `DiagonalPhase::s` and `DiagonalPhase::t`
- **THEN** the values are exactly `1/2`, `1/4` and `1/8` turns, matching `diag(1, −1)`, `diag(1, i)`
  and `diag(1, e^{iπ/4})`, and `phase_at(0)` is exactly `0` for all three

#### Scenario: A whole turn is the same phase

- **WHEN** two representatives induce phases of `3/2` and `1/2` turns on one basis state
- **THEN** the check accepts, because the difference is a whole turn, where an equality test on the
  two rationals would report a change of phase that did not happen

#### Scenario: No tolerance member reaches the report

- **WHEN** the report of `check_class_invariance` is inspected
- **THEN** it carries the exact residual, the examined count and the first failing pair, with no
  `Tolerance<R>` member and no epsilon anywhere on its path

### Requirement: The verdict is reached without materialising a unitary

`check_class_invariance` SHALL decide its predicate by 𝔽₂ and rational arithmetic over the supports
involved, and SHALL NOT build, simulate or compare `2ⁿ × 2ⁿ` operators.

The geometric-QEC consumer's subject is the 32-qubit code of `square_torus(4)`. `SimQpu::sample`
refuses a circuit above 24 qubits and sits behind the `qpu` feature, so comparing unitaries decides
nothing about that subject. The atom decomposition of `{γ, γ ⊕ b, γ̃}` is bounded by the union of
those three supports, and `ATOM_ENUMERATION_CAP` at `1 << 22` bounds what the check attempts.

#### Scenario: A 32-qubit code is decided in the default build

- **WHEN** `Z̄`, `S̄` and `T̄` are checked over the code derived from
  `LatticeComplex::<2, FloatType>::square_torus(4)`
- **THEN** each returns a verdict in the default build with no `qpu` feature, no sampler and no state
  vector, and the report names how many states the enumeration visited

#### Scenario: A wide code fails loudly

- **WHEN** the atom occupancy enumeration for one pair would visit more states than
  `ATOM_ENUMERATION_CAP`
- **THEN** the check returns `QuantumError::CalculationError` naming the state count and the cap, so
  the cost is reported and no run hangs

### Requirement: A class-invariance failure is reported as a witness and two counts, never as a margin

`check_class_invariance` SHALL report a witness identifying the failure and two counts, and SHALL NOT
report a margin. The witness names the shift that failed and the three block occupancies of the basis
state that witnessed it; those occupancies determine both overlap counts, so they determine the
phase, so they are the counterexample. The counts are the shifts examined and the states visited.

**A margin is withheld because the quantity has no order.** The residual is a phase difference in
ℝ/ℤ carried as an exact `Rational`. A residual of a half turn is not worse than one of an eighth
turn; they are different failures rather than graded ones. Reporting a magnitude would invent an
ordering with no physical content, and a caller would eventually sort by it.

This follows the convention the crate already applies to its other exact check.
`CausalStructure::check_c3_exclusion` reports no margin either: it returns
`NotFaithfullyRepresentable` naming the inputs and outputs that witness the C₃ obstruction. A check
whose quantity is a norm reports a margin, because norms are ordered; a check whose quantity is exact
reports the obstruction.

The examined count discharges the vacuous-pass obligation, and the visited count reports the work the
enumeration actually did, which is what makes an expensive code visible before it is a hang.
Enumeration MAY stop at the first failing state; whichever path runs, both counts SHALL describe what
was actually visited rather than a total the run never reached.

#### Scenario: A failing gate names its witness and both counts

- **WHEN** `check_class_invariance` rejects a gate
- **THEN** it names the failing shift and the three occupancies, reports how many shifts were
  examined and how many states were visited, and reports no margin

#### Scenario: The witness reproduces the failure without the checker

- **WHEN** the reported occupancies are read back and the phase recomputed from them directly
- **THEN** the result is the same non-integral phase, so the report can be checked by hand

#### Scenario: A vacuous pass is visible as one

- **WHEN** a gate is checked against an empty set of shifts
- **THEN** it reports `holds == true` with `tested == 0`, which a caller can distinguish from a pass
  that examined something

#### Scenario: Stopping early still reports honest counts

- **WHEN** the enumeration rejects on the first failing state
- **THEN** the counts are the shifts and states actually visited up to that point, rather than a
  total the run did not reach
