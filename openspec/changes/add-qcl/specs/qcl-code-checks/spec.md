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
consulted by this check, because they fix which superpositions are code states and a diagonal phase
does not see a superposition; they are carried for the Pauli predicate's normalizer check.

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
the X-stabilizer generators as a basis of `im δₖ₋₁`, SHALL carry both, and SHALL expose each through
an accessor. There SHALL NOT be a second constructor, and there SHALL NOT be an error path for a
basis carrying no stabilizers.

`from_complex` already holds the chain complex, which is the only thing a stabilizer basis needs, so
requiring a caller to supply the generators separately would add a way to build a basis that answers
the class-invariance question wrongly. Deriving them removes that state rather than validating it.

The generators SHALL be an image basis rather than the raw `∂ₖ₊₁` columns. The columns carry one
dependency per `(k+1)`-cycle, and a dependent generating set inflates the span test's cost without
changing its answer.

`is_logically_trivial` SHALL check its normalizer precondition against the carried generators and
SHALL return `QuantumError::NotInNormalizer` naming the offending generator when it fails. B.1's
criterion decides triviality *of a logical operator*; an operator outside the normalizer commutes
with the logical generators without acting trivially, because it leaves the code space instead. The
shipped doc block recorded that the precondition was stated rather than checked because no
stabilizer group was carried. One is carried now, and the check is two more loops of
`Gf2Chain::inner`: a Pauli `(x, z)` is in the normalizer iff `⟨x, s⟩ = 0` for every Z-generator `s`
and `⟨z, t⟩ = 0` for every X-generator `t`. Leaving the precondition stated once the data is present
would be a silent wrong answer.

#### Scenario: The generators are an independent basis of the boundaries

- **WHEN** the 3×3 simplicial torus is read at grade 1, where `β₂(T²) = 1` over 𝔽₂ gives exactly one
  dependency among the `∂₂` columns
- **THEN** the stabilizer count is one fewer than the column count, and the rank equals the count

#### Scenario: Every stabilizer induces a trivial gate

- **WHEN** each stabilizer generator is read as `Z̄`, `S̄` and `T̄`
- **THEN** `is_diagonal_trivial` accepts all three, which is Haruna Eq. (3.21) for the whole family
  rather than for `S̄` alone

#### Scenario: The Pauli path is unchanged on the normalizer

- **WHEN** a `LogicalBasis` decides `is_logically_trivial` for the `Z̄` Paulis over the torus's
  homology generators and its `∂₂` columns
- **THEN** the answers match `is_diagonal_trivial` on the corresponding `DiagonalPhase::z` gates,
  which is the agreement `test_the_diagonal_predicate_agrees_with_the_pauli_one_on_z` already asserts

#### Scenario: A Pauli outside the normalizer is refused rather than misjudged

- **WHEN** `is_logically_trivial` is given a single-qubit `X` on a qubit that a Z-stabilizer covers
- **THEN** it returns `NotInNormalizer` naming that stabilizer, rather than `Ok(true)`, which is what
  the commutation criterion alone would answer for an operator that leaves the code space

#### Scenario: The X-generators are an independent basis of the coboundaries

- **WHEN** the 3×3 simplicial torus is read at grade 1
- **THEN** the X-stabilizer count is `rank δ₀`, the rank equals the count, and every X-generator
  pairs to zero with every Z-generator, which is `∂₁ ∘ ∂₂ = 0` read through the pairing

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
nothing about that subject. The block decomposition of `{γ ∩ b, b \ γ, γ \ b}` is bounded by the two
supports, and `ATOM_ENUMERATION_CAP` at `1 << 22` bounds what the check attempts.

The consequence is stated so it is never overstated: the geometric-QEC path is **verified by exact
𝔽₂ and rational predicates, and not simulated**. No gate that consumer emits is ever run through
`SimQpu`, and the only dynamical evidence for any Haruna gate is the gate-alphabet identity tests on
small registers. That is sufficient because the checks are combinatorial and exact, and a claim
about this path SHALL NOT say "tested on the simulator".

#### Scenario: A 32-qubit code is decided in the default build

- **WHEN** `Z̄`, `S̄` and `T̄` are checked over the code derived from
  `LatticeComplex::<2, FloatType>::square_torus(4)`
- **THEN** each returns a verdict in the default build with no `qpu` feature, no sampler and no state
  vector, and the report names how many states the enumeration visited

#### Scenario: A wide code fails loudly

- **WHEN** the block occupancy enumeration for one shift would visit more states than
  `ATOM_ENUMERATION_CAP`
- **THEN** the check returns `QuantumError::CalculationError` naming the state count and the cap, so
  the cost is reported and no run hangs

#### Scenario: The verification claim names its kind

- **WHEN** the geometric-QEC consumer reports its results
- **THEN** it says the gates were verified by exact predicates and not simulated, and it names the
  simulator's 24-qubit cap against the code's 32

### Requirement: The class-invariance check covers the diagonal gates of Table 1, and says so

`check_class_invariance` SHALL be documented as covering the diagonal gates of Table 1 — `Z̄`, `S̄`,
`T̄`, `CZ̄`, `CS̄†`, `CC̄Z` and `C^{m−1}Z̄` — and SHALL NOT be described as covering "the Haruna gate
layer". `H̄` is emitted by that layer and is not decided by this stage.

"The Haruna gate layer" names the emitter, which produces every Table 1 gate. Read beside "every
diagonal gate", it implies the rational check covers the whole layer, and it does not: `H̄` is
neither a Pauli nor diagonal.

#### Scenario: Coverage is stated where the check is described

- **WHEN** a reader consults the documentation of `check_class_invariance`
- **THEN** it lists the seven diagonal gates it decides and states that `H̄` is decided by
  `check_clifford_action`

### Requirement: `check_clifford_action` decides `H̄` on the symplectic side

`check_clifford_action` SHALL propagate each logical Pauli generator through an emitted
`Vec<GateOp>` as a symplectic 𝔽₂ update on `(x, z)`, and SHALL decide, through `LogicalBasis`, that
the image of `Z̄(γ)` is logically equivalent to `X̄(γ̃)` and the image of `X̄(γ̃)` to `Z̄(γ)`, up to
phase. It SHALL NOT build a state vector and SHALL carry no register-width limit.

`H̄ = S̄(γ) · ∏H · S̄(γ̃) · ∏H · S̄(γ)` (Eq. 3.27) is built from `S`, `CZ` and `H`, all Clifford, so
it is a Clifford circuit, and Clifford conjugation of a Pauli is a stabilizer-tableau update the
crate already has the vector type for: `LogicalPauli<W>` is `(x, z)` over `Gf2Chain`. The
symplectic actions are `H: (x, z) ↦ (z, x)` on its qubit, `S: (x, z) ↦ (x, z ⊕ x)`, and
`CZ: (x, z) ↦ (x, z ⊕ x_swapped)`, with the phase tracked separately. This stage covers `Z̄`, `S̄`,
`CZ̄` and `H̄`; it cannot cover `T̄`, `CS̄†` or `CC̄Z`, which are non-Clifford and remain with the
diagonal check. Between the two stages every Table 1 gate is decided by exactly one exact predicate,
and `Z̄`, `S̄` and `CZ̄` are decided by both, which is the cross-check.

#### Scenario: The logical Hadamard swaps the logical Paulis

- **WHEN** `logical_hadamard(γ, γ̃)` is emitted for a class of the 3×3 torus and its dual
- **THEN** pushing `Z̄(γ)` through the program yields a Pauli `are_logically_equivalent` to `X̄(γ̃)`,
  and pushing `X̄(γ̃)` yields one equivalent to `Z̄(γ)`, each up to phase

#### Scenario: The diagonal Cliffords agree with the diagonal check

- **WHEN** `logical_s(γ)` and `logical_cz(γ₁, γ₂)` are decided by both stages
- **THEN** both accept, and the Clifford stage's image of `X̄(γ̃)` under `S̄` is `X̄(γ̃)` times
  `Z̄(γ)^{⟨γ, γ̃⟩}` up to phase, which is Eq. (3.20)'s commutation relation read on the symplectic side

#### Scenario: A non-Clifford program is refused, not misjudged

- **WHEN** `check_clifford_action` is given a program containing `T`, `Csdg` or `Ccz`
- **THEN** it returns a structured `QuantumError` naming the first non-Clifford gate, because the
  tableau update is undefined for it, rather than propagating through a gate it cannot represent

### Requirement: The emitted program carries its global phase

The program `logical_hadamard` emits SHALL carry the global phase it returns as an optional
`global_phase: Option<Complex<R>>` on the program, and the causal wrapper SHALL populate it rather
than drop it.

Table 1's Hadamard carries `e^{−iπ/4}`. A global phase is unobservable under computational-basis
measurement and becomes a relative, observable one the moment the gate is used as a controlled
operation, which is what the Appendix B arguments carry. `check_clifford_action` tests equivalence
up to phase because the symplectic form is phase-blind; carrying the phase beside the program is
what lets a later exact form of the test, or a controlled use, recover it.

#### Scenario: The wrapper keeps the phase

- **WHEN** the causal wrapper over `logical_hadamard` is called
- **THEN** the `PropagatingEffect` it returns carries the program with `global_phase` set to the
  `Complex<R>` the builder returned, and nothing about the phase is logged as dropped

#### Scenario: Diagonal programs carry no phase

- **WHEN** `logical_z`, `logical_s`, `logical_t`, `logical_cz` or `logical_multi_cz` emits a program
- **THEN** its `global_phase` is `None`, because Table 1 attaches no global phase to those rows

### Requirement: The non-Clifford builders report their tuple count and cap it

`logical_t` and `logical_multi_cz` SHALL report the number of pair and triple tuples they emit over,
and SHALL return a structured `QuantumError` naming the count and the cap when the count exceeds a
configurable cap, before allocating the tuple list.

`Gf2Chain::support_pairs` and `support_triples` materialise `C(w, 2)` and `C(w, 3)` tuples eagerly.
On a toric code `w` is the lattice extent and the cost is nothing; on the qLDPC family the 𝔽₂ rank
was fixed for, representatives can have weight in the tens to hundreds and `T̄` materialises `10⁵`
to `10⁶` triples per gate. The cap follows the pattern the code-space enumeration uses: cost is
reported before it is paid, and a run that would exceed it fails loudly rather than allocating.
Making the iterators lazy is the alternative and is not chosen for v1.

#### Scenario: A toric representative is under the cap

- **WHEN** `logical_t` is emitted for a homology generator of `square_torus(4)`
- **THEN** the program is emitted, and the tuple count it reports is `C(w, 2) + C(w, 3)` for the
  generator's weight `w`

#### Scenario: A wide representative fails loudly

- **WHEN** `logical_t` is asked for a chain whose weight puts `C(w, 3)` above the configured cap
- **THEN** it returns a structured `QuantumError` naming the count and the cap, and no tuple list is
  allocated

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
