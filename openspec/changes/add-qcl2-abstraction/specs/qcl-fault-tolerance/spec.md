<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: A fault set is a query signature over the low-level model, counted and capped

`FaultSet` SHALL be a finite signature of low-level queries, each a `Query::Fault` inserting a
Pauli error on named wires after a named node of the circuit (or before its first box), with
constructors `pauli_weight(locations, after, t, cap)`, `declared(&[Fault])` and
`from_dem(mechanisms, after)` taking the Pauli supports of a detector error model's mechanisms, and
`pauli_weight` on `n` locations SHALL count `C(n, t) · 3^t` queries in checked `u64` arithmetic and
SHALL refuse above the cap before allocating, naming the count and the cap.

A fault is a comb in the paper's sense, a general intervention that is not a Do-query. The count
is the same exponential D7 of `add-qcl` caps for the design cover; the default weight is one, and
realistic sets come from a detector error model rather than from enumeration.

#### Scenario: Weight-one Paulis on the small torus

- **WHEN** `FaultSet::pauli_weight(1)` is built over the 8 locations of the `[[8,2,2]]` circuit
- **THEN** it holds `24` queries, one per `(location, Pauli)`, its count reads `24`, and it displays
  as `pauli_weight(1) (24 faults)`

#### Scenario: A set above the cap is refused

- **WHEN** `FaultSet::pauli_weight(3)` is built over `n` locations such that `C(n, 3) · 27` exceeds
  the cap
- **THEN** construction returns `QuantumError::CalculationError` naming the count and the cap, and
  no queries are allocated

### Requirement: Faults propagate exactly through every Table 1 gate, with no term cap

The fault propagator SHALL carry a Pauli fault through a Clifford layer by the shipped
`clifford_conjugate` rule and through a diagonal Table 1 gate by conjugation in the algebra of the
gate's logical `Z̄(γᵢ)` operators, SHALL represent the diagonal gates as `GaugeFieldGate` values
(the blocks `γ₁, …, γ_m` as `Gf2Chain`s and the phase function on `{0,1}^m` as `2^m` values of
`Turns`), and SHALL refuse a program outside that normal form with
`QuantumError::NoPropagationNormalForm` naming the layers, rather than expanding it to a cap.

Haruna defines every diagonal logical gate as `O_k(γ₁, …, γ_m) = exp(iπ/2^{k−1} · p₁⋯p_m)` with
`pᵢ = (I − Z̄(γᵢ))/2` (Eq. 3.63; `S̄` 3.14, `CZ̄` 3.37, `C^{m−1}Z̄` 3.49, `T̄` 3.56) and derives the
physical decomposition from it. Conjugating a Pauli `P = X^a Z^b` through such a gate `G` gives
`P · (P† G P) G†`, where `P† G P` is `G` with each `Z̄(γᵢ)` replaced by `(−1)^{⟨a, γᵢ⟩} Z̄(γᵢ)`. The
remainder is a `GaugeFieldGate` on the same blocks, computed from `m` inner products through
`Gf2Chain::inner`, and has at most `2^m` Pauli terms whatever the representative weight. Z-type
faults commute with every diagonal gate. The physical program of the gate is never materialised by
the propagator, so the emitter's tuple cap on `logical_t` does not bear on the fault analysis.

#### Scenario: A Clifford layer keeps one term

- **WHEN** an `X` fault on qubit `q ∈ γ` is propagated through `S̄(γ)` with `γ` of weight 3
- **THEN** the result is the single Pauli `X_q Z̄(γ)` up to phase, which is `X_q` times the
  remainder `Z̄(γ)` of `S̄` under a flipped parity

#### Scenario: `T̄` leaves a two-term remainder at every weight

- **WHEN** an `X` fault on `q ∈ γ` is propagated through `T̄(γ)` for `γ` of weight 3, 4 and 5
- **THEN** the remainder is `exp(±iπ/4 · Z̄(γ))` in each case, its Pauli terms are `I` and `Z̄(γ)`
  with coefficients of modulus `1/√2`, and the term count does not change with the weight

#### Scenario: An even-overlap fault does not spread

- **WHEN** `X_q X_r` with `q, r ∈ γ` is propagated through `T̄(γ)`
- **THEN** `⟨a, γ⟩ = 0`, the remainder is the identity, and the result is `X_q X_r`

#### Scenario: A Z-type fault passes through unchanged

- **WHEN** `Z_q` is propagated through `T̄(γ)` and through `CZ̄(γ₁, γ₂)`
- **THEN** the result is `Z_q` in both cases

#### Scenario: A program outside the normal form is refused by name

- **WHEN** a user program applies `T̄(γ)`, then transversal `H` on `γ`, then `T̄(γ)` again
- **THEN** the propagator returns `NoPropagationNormalForm` naming the second non-Clifford layer,
  and allocates no expansion

### Requirement: Fault tolerance is the naturality check over the enlarged signature

`check_fault_tolerance` SHALL run the naturality square over the abstraction's signature enlarged
by every fault in the set, SHALL report per-fault residuals, the worst, the count examined and the
witnessing fault, and on the exact path SHALL decide each fault by whether the remainder's phase
function is constant on `{0,1}^m`, as a comparison of `Turns`, and whether the propagated Pauli
has no more weight than the fault, so that a recovery built for the set's weight still corrects it.
On the exact path the check is `CodeAbstraction::check_fault_tolerance(gate, set)`; on the numeric
path it is `Abstraction::check_fault_tolerance(set, caps)` over a circuit model, each fault as a
`Query::Fault` against the high-level `Io`. The two paths answer neighbouring questions: whether the
gate spreads the fault, and whether `τ` absorbs the spread fault; they agree when `τ` corrects every
error of the set's weight.

After the fault's own Pauli is recovered, the remainder acts on the code space as
`exp(2πi · Δg(p̂))` with `p̂` the logical parity operators. Because the `γᵢ` are independent
homology classes, every `Z̄(γ_S)` with `S ≠ ∅` is a non-trivial logical operator, so the remainder is
a non-trivial logical unitary exactly when `Δg` is not constant. The witness is the flipped parity
pattern and the remainder's phase table. The report carries a witness rather than a margin for the
reason D10 of `add-qcl` gives: which fault broke the square is the information. Each record carries
the `SemanticsPath` that decided it, and on Table 1 that path is `Exact`.

#### Scenario: A transversal gate under weight-one noise passes

- **WHEN** `check_fault_tolerance` runs on `Z̄(γ)` of the `[[18,2,3]]` torus under
  `pauli_weight(1)`
- **THEN** every fault propagates to a single-qubit Pauli with the identity remainder, the report
  accepts with examined count `3 · 18`, and every record reads `SemanticsPath::Exact`

#### Scenario: A non-constant remainder is a logical fault, and is named

- **WHEN** `check_fault_tolerance` runs on `T̄(γ)` of the `[[18,2,3]]` torus under `pauli_weight(1)`
- **THEN** every `X` or `Y` fault on `γ` rejects with the remainder `exp(±iπ/4 · Z̄(γ))` as witness,
  its phase table `[1/8, 7/8]` and two Pauli terms, every `Z` fault and every fault off `γ`
  accepts, and the report's examined count is `3 · 18`

#### Scenario: A Clifford gate that spreads a fault is named by the weight

- **WHEN** `check_fault_tolerance` runs on `H̄(γ)` of the `[[18,2,3]]` torus under `pauli_weight(1)`
- **THEN** every rejected fault has a constant remainder and a propagated Pauli of weight above
  one, and the witness names the weight and whether the carried operator is a non-trivial logical
  operator

#### Scenario: A traced-out fault is tolerated on the numeric path

- **WHEN** `check_fault_tolerance` runs on a two-wire circuit whose `τ` traces the second wire,
  under a declared set with faults on both wires
- **THEN** the faults on the traced wire have residual zero and the faults on the kept wire a
  positive residual, every record reads `SemanticsPath::Numeric`, and the witness names the norm

#### Scenario: An empty fault set is vacuous

- **WHEN** `check_fault_tolerance` runs with `FaultSet::declared(&[])`
- **THEN** the report's verdict is `Vacuous` with examined count zero

### Requirement: The Haruna filter's answer is derived and then computed

The Haruna filter SHALL run `check_fault_tolerance` under `pauli_weight(1)` on each Table 1 gate
over a CSS code, SHALL output the subset that holds, and SHALL label every verdict `Exact`.

The answer follows from Eq. (3.63) for every CSS code and every representative weight: `Z̄` and `X̄`
do not spread, and every other Table 1 gate fails a single X-type fault on its support because its
remainder is a non-trivial logical operator (`i · Z̄(γ)` for `S̄`, phase table `[1/4, 3/4]`;
`exp(±iπ/4 Z̄(γ))` for `T̄`, table `[1/8, 7/8]`; `Z̄(γ₂)` for `CZ̄` under a fault on `γ₁`, table
`[0, 0, 1/2, 1/2]`; and the Clifford image through the tableau for `H̄`, a Pauli of weight above
one). That derivation is
the provenance of the test's expected values, as the anti-circularity protocol asks; the filter
computes the verdicts and the test compares them to it.

#### Scenario: The filter agrees with the derivation on both torus fixtures

- **WHEN** the filter runs on `[[18,2,3]]` and on `[[32,2,4]]`
- **THEN** `Z̄` and `X̄` hold under weight-one faults, `S̄`, `H̄`, `CZ̄` and `T̄` do not, each rejection
  names its witness, and every record reads `Exact`

#### Scenario: The filter's cost does not grow with the representative weight

- **WHEN** the filter runs `T̄` on a representative of weight `w` for `w = 3` and `w = 4`
- **THEN** the number of remainder terms examined is two in both cases

### Requirement: The fault-tolerance claim is narrowed, not removed

Every fault-tolerance report SHALL state its fault set, its residual or remainder and its semantics
path, and the crate SHALL make no claim of a threshold, a distance or asymptotic suppression.

#### Scenario: The report names its fault set

- **WHEN** a `check_fault_tolerance` report is displayed
- **THEN** it names the fault set's constructor and count, the worst residual, the path per gate,
  and carries no field for a threshold or a distance
