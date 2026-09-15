<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Why

QCL v1 answers two questions separately: whether a factorization of a physical process operator is
Markov, decomposable and CPTP (`qcm`), and whether a physical gate program acts correctly on a code
space (`qcode`). Nothing in the crate relates the two. The physical-to-logical map is not a type,
no stage asks whether a physical factorization induces a logical one, and fault tolerance,
concatenation and decoder validation are all outside what the pipeline can express. The archived
`add-qcl` change recorded this as a Non-Goal and reserved the name `qcl-abstraction` for it.

Lorenz and Tull's *Causal and Compositional Abstraction* (arXiv:2602.16612, February 2026) supplies
the missing object: an abstraction between two compositional models is a type alignment `(π, τ)`
together with a query map, and it is correct exactly when every square
`τ ∘ ⟦π(Q)⟧_L = ⟦Q⟧_H ∘ τ` commutes. The category QC of controlled quantum instruments (the paper's
Example 57) contains both quantum circuits and classical causal models, so one definition covers a
code as an abstraction from physical to logical qubits, a decoder as an abstraction from a physical
circuit to a classical detector model, and their composites. The road map in
[`qcl2-roadmap.md`](../../notes/quantum/qcl2-roadmap.md) stages that construction in six phases.
Its verification against the tree,
[`qcl2-roadmap-verification.md`](../../notes/quantum/qcl2-roadmap-verification.md), found fifteen
corrections, three of which change exit criteria: the Choi-operator formulation of the naturality
check cannot be formed on the fixtures the road map names (17 TB on `[[18,2,3]]`), the seven live
`qcl-*` specifications are empty because the archive step never merged them, and the Haruna filter's
non-Clifford gates are reachable by neither the tableau nor the simulator. This change is the road
map as corrected by that register.

## What Changes

- **The circuit as the stored object.** `CircuitModel<R>` is a compositional model in QC: a
  `QuantumCircuit` with typed wires, noise boxes, a box-to-node grouping and declared outputs, with
  two semantics functors. The numeric one carries the program at the Kraus level and forms one
  Choi operator, for the composite `2^n → 2^k` channel, under two caps. The exact one carries Pauli and Clifford programs as symplectic 𝔽₂ data and
  diagonal gates as rational phases, with no width limit. `Dilation` marginalises a circuit to
  `(ProcessFactors, FactorSupports)` under a fixed leg convention, and `.over_circuit` joins
  `.over_model` on `QclBuilder`. A bare process operator still validates as in v1 and cannot enter
  an abstraction (`NoCompositionalModel`).
- **The abstraction object and its check.** `TypeAlignment` (each `τ_X` CPTP, each with a section
  `E_X` witnessing `τ_X ∘ E_X = id`), `QuerySignature` (`Io`, `Open(S)`, `Inc(S₁…Sₙ)` on
  parallelisable sets, `Observe(O)`), `Abstraction<L, H>` as a downward abstraction, and
  `check_naturality` as a `Check<R>` that reports its residual, its norm, its amplification factor,
  its semantics path and the count of queries examined. An abstraction whose squares commute to
  residual `ε` is an ε-abstraction, a definition this crate makes and the paper does not.
- **The structural precheck and the code as an abstraction.** `check_alignment_structure` computes
  Definition 49's `α(X)` and the simple, extra-simple and full predicates on the two DAGs. It is
  Theorem 51 when both models are classical and a documented necessary condition when the low-level
  model is quantum. `CodeAbstraction` builds the strict abstraction of a CSS code from its
  `LogicalBasis`, and the generation regression pins its exact-path naturality check to
  `check_class_invariance` and `check_clifford_action` on every gate and every fixture.
- **Fault sets and the fault-tolerance predicate.** `FaultSet` (`pauli_weight(t)`, `declared`,
  `from_dem`) enlarges the low-level signature; `check_fault_tolerance` is `check_naturality` over
  it, reporting per-fault residuals and the witnessing fault. Faults propagate exactly and without
  a cap through every Table 1 gate: Clifford layers through the tableau, diagonal gates through the
  algebra of their logical `Z̄(γᵢ)` operators (Haruna Eq. 3.63), in which a propagated fault has at
  most `2^m` Pauli terms for a gate on `m` logical qubits, whatever the representative weight. The
  Haruna filter runs it over Table 1; its answer under weight-one faults follows from that algebra
  and is confirmed by the computation.
- **Composition.** `Abstraction::compose` applies
  `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` with both constants the Frobenius-induced norms of `τ₂` and
  `τ₁`, computed and recorded, exact composition being the paper's Proposition 17. Three consumers are abstraction
  chains: concatenated codes, code switching, and a distillation round as the non-strict
  quantum-to-quantum case the paper leaves open.
- **The decoder as an abstraction.** `DemModel` imports a detector error model as a classical
  causal model over a `CausaloidGraph`; `DecoderAbstraction` takes the decoder as a black-box
  `τ` and validates it by naturality under openings; the logical attribution query ranks the
  low-level faults whose squares fail.
- No breaking change. Every addition is a new name beside an existing one.

## Capabilities

### New Capabilities

- `qcl-circuit-model`: `CircuitModel<R>`, its two semantics functors and their caps, `Dilation`
  with the leg convention, `.over_circuit`, and the `NoCompositionalModel` boundary.
- `qcl-abstraction`: `TypeAlignment` with sections, `QuerySignature`, `Abstraction<L, H>`,
  `check_naturality` with its two paths and its report, `check_alignment_structure` with its
  scoped claim, `CodeAbstraction` and the generation regression, `Abstraction::compose` and the
  ε-law.
- `qcl-fault-tolerance`: `FaultSet`, the `GaugeFieldGate` carrier and exact propagation through
  Table 1, `check_fault_tolerance`, the Haruna filter and the narrowed fault-tolerance claim.
- `qcl-decoder-abstraction`: `DemModel`, the `dem` feature and its Stim text import,
  `DecoderAbstraction`, and the logical attribution query.

### Modified Capabilities

- `qcl-pipeline`: the builder gains a fourth subject constructor, `.over_circuit`, and `validate`
  gains the abstraction stages. The live `qcl-pipeline` specification is empty (verification V-1),
  so the delta is written as `ADDED` requirements and a task restores the seven live `qcl-*`
  specifications from the archived `add-qcl` deltas before implementation begins.

## Impact

**Code.** New modules under `deep_causality_quantum/src/types/`: `circuit_model/`, `abstraction/`,
`fault/` and `dem/`, one type per module, with the test tree mirroring them. They consume the
shipped `qgates` (Choi operators, `embed_on_legs`, `apply_kraus`), `carriers` (`Channel`),
`decision` (`Check<R>`, `CheckReport<R>`), `qcode` (`LogicalBasis`, `clifford_conjugate`,
`DiagonalPhase`), `qcm` (`ProcessFactors`, `Hypothesis::compose`) and `qpu/circuit` (`GateOp`,
`QuantumCircuit`) layers without reimplementing them. Five kernels are new: density-matrix
evolution of a `GateOp` program with noise boxes at the Kraus level, the `GaugeFieldGate`
propagator, the Choi of a `2^n → 2^k` composite, the Frobenius-induced channel norms of the ε-law,
and a detector-error-model parser. Nothing is added to the unified math stack; the design's
foundation section maps each kernel to a shipped primitive.
Each is a numeric kernel and follows the unified-math TDD protocol (anti-circularity, corner-case
rows A to K, defect audit, `cargo mutants`).

**Unified math.** No new dependency. The layer stands on `deep_causality_tensor` (`CausalTensor`),
`deep_causality_linear` (the Hermitian eigendecomposition under the CP check and the 𝔽₂ matrices
under `LogicalBasis`), `deep_causality_homology` (`Gf2Chain`, `ChainComplex`),
`deep_causality_num_rational` (`Turns`), `deep_causality_algebra` (`RealField`) and
`deep_causality_num` (the lifts). Every check stays generic in `R: RealField`, and the consumers
run at `f32`, `f64` and `Float106` as the three v1 consumers do.

**Features.** `CircuitModel`, `TypeAlignment`, `Abstraction`, `check_naturality`,
`check_alignment_structure`, `CodeAbstraction` and `FaultSet` need `alloc` only and compile in the
default and `no-std` builds. `Dilation`, `.over_circuit` and `DemModel` are `qcm`-gated because
`ProcessFactors` and `CausaloidGraph` are. The Stim text import sits behind a new `dem` feature,
enabled in `BUILD.bazel` beside `qpu`.

**APIs.** Additive. `QclBuilder` gains `.over_circuit`; `Validate` gains `check_alignment_structure`
and `check_naturality`; `QuantumError` gains `NoCompositionalModel`, `NaturalityDimensionExceeded`,
`KrausFamilyExceeded`, `NoPropagationNormalForm`, `NotParallelisable` and `SectionNotInverse`. release-plz derives the
version bump from the commit messages; nothing edits `CHANGELOG.md` or the version by hand.

**Verification.** Proposition 17 in the exact case goes in `lean/DeepCausalityFormal/Quantum/
Abstraction.lean`, bound through `lean/THEOREM_MAP.md` to the Rust generation regression. The
ε-law and the Frobenius-to-diamond factor are Rust with their derivations in the docstrings, as
G-16 did for `√(d_B)`. Every other check names its witness or says it has none.

**Documentation.** The road map is not edited; the verification register is its errata and both are
cited from the design. `qcl-design-note.md` §9 gains a QCL-2 row per shipped group. The archived
`add-qcl` proposal's Non-Goal on relating the `qcm` and `qcode` subjects is superseded by
`qcl-abstraction`, and the fault-tolerance Non-Goal is narrowed, not removed: the claim is "the
naturality square holds under fault set `F` to residual `ε`" and nothing about thresholds,
distances or asymptotic suppression.

**Out of scope.** Implementing a decoder; diamond-norm evaluation by SDP; claiming exact
naturality from a measured residual; making a general Barrett–Lorenz–Oreshkov process operator a
compositional model, which the paper names as future work; cyclic structures; device models.
