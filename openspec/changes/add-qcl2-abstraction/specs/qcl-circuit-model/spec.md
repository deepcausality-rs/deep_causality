<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: A circuit model is a compositional model in QC

`CircuitModel<R>` SHALL be a compositional model of an open DAG in the sense of Lorenz & Tull
Definition 59: a signature of boxes and typed wires together with a semantics for each box, from
which the induced DAG of Example 61 is derived and not stored.

The boxes are encoders (classical input to quantum), unitaries (a `GateOp` program on named
wires), channels (a `Channel` on named wires, the noise boxes), Kraus boxes (a CPTP map given by
its Kraus operators alone, for a wide box such as a code's encoder unitary whose Choi operator the
`Channel` carrier could not hold), instruments (a controlled Kraus family with a classical outcome
wire) and measurements (quantum to classical). Wires are quantum,
carrying a Hilbert dimension, or classical, carrying a finite outcome set. A box-to-node grouping
names which boxes form one vertex of the model's DAG, so that a gate-level circuit can be viewed as
a model of a coarser DAG, which the paper calls a strict component-level abstraction (Example 63).
Declared outputs are a subset of the vertices. The `QuantumCircuit` and `GateOp` types are consumed
as they ship; `CircuitModel` adds the wire types, the noise boxes, the grouping and the outputs.

#### Scenario: The induced DAG follows the wires

- **WHEN** a `CircuitModel` is built from an encoder on qubits `{0, 1}`, a unitary box `U` on
  `{0, 1}`, a unitary box `V` on `{1, 2}` and measurements on `{0, 2}`, with each box its own node
- **THEN** `induced_dag()` has a vertex per encoder input, per unitary and per measurement output,
  an edge for each internal wire, `U → V` present because wire `1` leaves `U` and enters `V`, and
  no edge `U → measurement(2)`

#### Scenario: A malformed wiring is refused at construction

- **WHEN** a box names a wire whose dimension disagrees with the box's declared dimension, or two
  boxes both write the same wire without a box between them
- **THEN** construction returns `QuantumError::DimensionMismatch` naming the wire and both boxes,
  and no `CircuitModel` value is produced

#### Scenario: The grouping is a partition of the boxes

- **WHEN** the grouping leaves a box in no node or in two nodes
- **THEN** construction returns `QuantumError::CalculationError` naming the box

### Requirement: A circuit model has two semantics functors, and every result names the one that decided

`CircuitModel<R>` SHALL expose an exact semantics for programs the stabilizer and phase-polynomial
formalisms cover and a numeric semantics for every program, and every value derived from either
SHALL carry `SemanticsPath::{Exact, Numeric}`.

The exact semantics carries a Pauli or Clifford program as its symplectic action through the
shipped `clifford_conjugate`, and a diagonal Table 1 gate as a `GaugeFieldGate`, the blocks as
`Gf2Chain`s and the phase function on the `m` logical parities as exact `Turns`, which is
`DiagonalPhase` generalised to `m` blocks. It has no register-width limit. The numeric semantics
carries the program at the Kraus level: each gate's unitary is embedded on the register through the
shipped `embed_on_legs` and multiplied into the running Kraus family, a noise box multiplies the
family out by its own operators, and the Choi operator is formed once, for the composite channel
from the declared inputs to the declared outputs, through the shipped `choi_from_kraus`. The Choi
of the program alone is never formed. A `Channel`-boxed noise model is reachable by the numeric
semantics only.

#### Scenario: A Clifford program is decided exactly on a wide register

- **WHEN** the exact semantics is asked for the image of `Z̄(γ)` under the `H̄` program on the
  `[[32,2,4]]` torus
- **THEN** it returns a `LogicalPauli` through `clifford_conjugate`, forms no matrix, and the
  result carries `SemanticsPath::Exact`

#### Scenario: A program with a noise box is numeric

- **WHEN** a model containing a `Channel` box is asked for the image of a Pauli under the exact
  semantics
- **THEN** it returns `QuantumError::CalculationError` stating that a channel box has no exact
  semantics, and the numeric semantics answers the same question with `SemanticsPath::Numeric`

### Requirement: The numeric semantics is capped by entry count and by Kraus family size before allocating

The numeric semantics SHALL compute the entry count of what it would allocate, the working storage
of a program evaluation and the `2^(2n + 2k)` entries of a composite Choi operator from `n` to `k`
qubits where one is formed, and SHALL refuse above a cap with
`QuantumError::NaturalityDimensionExceeded { n, k, entries, cap }` before allocating; it SHALL
compute the Kraus family size `∏ kᵢ` over the program's noise boxes and SHALL refuse above a second
cap with `QuantumError::KrausFamilyExceeded { operators, cap }` before allocating; and it SHALL
report the entries it formed on success.

The default caps are `2^24` entries and `2^12` operators. Both counts are ℕ on `NumberType` with
checked products, so a register or a family that would overflow the count reads as above any cap
rather than as small. A fault-set query inserts one error channel of at most four operators, so the
fault path stays far below the second cap.

#### Scenario: The 18-qubit torus is refused

- **WHEN** the numeric semantics is asked to evaluate an 18-qubit program with two output qubits
  under the default cap
- **THEN** it returns `NaturalityDimensionExceeded { n: 18, k: 2, entries: 2^36, cap: 2^24 }`, the
  working storage of `2^18` state vectors of `2^18` amplitudes it would allocate, before allocating;
  the composite Choi of `2^40` entries is refused by the same variant at the point it would be
  formed

#### Scenario: The small torus is formed and its cost reported

- **WHEN** the numeric semantics forms the composite Choi for `n = 8`, `k = 2`
- **THEN** it returns the `2^20`-entry operator and a report whose examined count is `2^20`

#### Scenario: A count that would overflow is above any cap

- **WHEN** `n + k` is large enough that `2^(2n + 2k)` does not fit the count type
- **THEN** the check reports `NaturalityDimensionExceeded` with `entries` saturated, not a wrapped
  small number

#### Scenario: A program with many noise boxes is refused by family size

- **WHEN** a program carries thirteen two-operator noise boxes, so `∏ kᵢ = 2^13`
- **THEN** the numeric semantics returns `KrausFamilyExceeded { operators: 2^13, cap: 2^12 }` and
  allocates no family

#### Scenario: A unitary program's composite is cheap

- **WHEN** the composite Choi of a noiseless 8-qubit program followed by a four-operator decoder
  `τ` is formed
- **THEN** it is built from the four operators `K_j U` through `choi_from_kraus`, has `2^20`
  entries, and no operator of `2^32` entries is formed on the way

### Requirement: The dilation marginalises a circuit under a fixed leg convention

`Dilation` SHALL map a `CircuitModel` to `(ProcessFactors<R>, FactorSupports)` by the
Barrett–Lorenz–Oreshkov construction, with each node's leg of dimension `d_in · d_out` declared
through `set_leg_dim`, the input index outer and the output index inner, and each factor
`ρ_{A|Pa(A)}` embedded as the identity on `A`'s output half and on every parent's input half.

The flat convention `support(A) = {A} ∪ Pa(A)` names one leg per node and cannot distinguish the
input space from the output space that BLO's factor acts on. The convention above makes the
commutation check run on the operators BLO's proposition says commute, and the DAG the supports
encode through `Hypothesis::structure_from_supports` is the circuit's induced DAG. `Dilation` is
`qcm`-gated because `ProcessFactors` is.

#### Scenario: The dilation of a circuit is Markov for its induced DAG

- **WHEN** a two-node unitary circuit with a broken wire between the nodes is dilated and
  `quantum_markov_check_report` runs on the result under `CommutatorTolerance::new()`
- **THEN** every intersecting pair commutes within Q-TOL, the report's factorization reads
  `Rederived`, and the supports encode the edge the wire induces

#### Scenario: The leg dimension is the product of the halves

- **WHEN** a node has a qubit input and a qubit output
- **THEN** `FactorSupports::declared_leg_dim` on its leg reads `4`, and the factor's shape validates
  against the product of its support's leg dimensions

#### Scenario: A glued circuit's dilation is the induced factorization

- **WHEN** two `CircuitModel` values are glued along a shared wire and the composite is dilated
- **THEN** the result is one factorization whose Markov check runs on factors of provenance
  `Rederived`, and no `CertificateNotInherited` is produced, because the induced factorization was
  constructed rather than inherited

### Requirement: A model without a dilation validates and stops

A configuration built with `.over_model(graph, factors, supports)` SHALL validate as in v1 and
SHALL be refused by every abstraction constructor with `QuantumError::NoCompositionalModel`
naming the reason.

The paper says a general BLO process operator is not a compositional model (Example 62, §8). A
caller who holds only the marginal keeps v1's checks and cannot enter an abstraction, and the
error says why rather than failing on a missing field.

#### Scenario: A bare process operator cannot enter an abstraction

- **WHEN** `Abstraction::new` is given a low-level side that is a `ModelSubject` rather than a
  `CircuitModel`
- **THEN** it returns `NoCompositionalModel` stating that a process operator without its circuit is
  the marginal of a compositional model and not one itself, and the v1 `validate` stages on that
  subject are unchanged
