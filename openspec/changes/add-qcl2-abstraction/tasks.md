<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Ordering follows the road map's phases as corrected by `qcl2-roadmap-verification.md`, with group 0
ahead of them because nothing else validates against empty specifications. Every group ends with a
verification task, and no group is done until `bazel test //...` is green for it. Every new numeric
kernel (groups 1, 3, 4 and 6) follows the unified-math TDD protocol: literals with stated
provenance, corner-case rows A to K in the test module doc, the defect audit, then `cargo mutants`.
A commit message is prepared at each group boundary; nothing is committed by the agent.

## 0. Restore the live specifications

- [x] 0.1 Copy the seven archived `add-qcl` deltas into `openspec/specs/qcl-*/spec.md` as their
      live text, requirement counts 7, 11, 7, 7, 7, 9 and 9
- [x] 0.2 Verify: `openspec validate --specs` green; this change still validates with the
      `qcl-pipeline` delta as `ADDED`

## 1. The circuit model and its two semantics

- [x] 1.1 Add `CircuitModel<R>` under `types/circuit_model/`: boxes (encoder, unitary, channel,
      instrument, measurement), typed wires, the box-to-node grouping, declared outputs, and
      `induced_dag()` per Example 61; construction rejects mis-dimensioned wires, double writers
      and a grouping that is not a partition
- [x] 1.2 Add the exact semantics: a Clifford program as its symplectic action through
      `clifford_conjugate`, a diagonal Table 1 gate as a `DiagonalPhase`; `SemanticsPath::Exact`
- [x] 1.3 Add the numeric semantics kernel at the Kraus level: each gate's unitary embedded through
      `embed_on_legs` and multiplied into the running family, each `Channel` box multiplying the
      family out, and the Choi of the composite `2^n → 2^k` channel formed once through
      `choi_from_kraus`; the program's own Choi is never formed; `SemanticsPath::Numeric`
- [x] 1.4 Add the two caps on the numeric path, both counted on `NumberType` with checked
      products and refused before allocating: `NaturalityDimensionExceeded { n, k, entries, cap }`
      at `2^24` entries and `KrausFamilyExceeded { operators, cap }` at `2^12` operators; report the
      examined count on success
- [x] 1.5 Add the `[[8,2,2]]` fixture from `LatticeComplex::<2, _>::square_torus(2)` (confirmed
      valid: `β = (1, 2, 1)`, `∂₁∂₂ = 0`, weight-2 representatives) and the hand-built `[[4,2,2]]`
      chain complex to `utils_tests`, each with its derivation in the module doc
- [x] 1.6 Verify: the numeric kernel against `apply_kraus` on one qubit and against a hand-computed
      two-qubit `CZ` Choi; the 18-qubit request refused with the exact entry count; both semantics
      agree on `Z̄` and `H̄` over the small fixture; defect audit and `cargo mutants` on the kernel

## 2. The dilation and the circuit subject

- [x] 2.1 Add `Dilation` under `qcm`: leg dimension `d_in · d_out` per node through `set_leg_dim`,
      input outer and output inner, `ρ_{A|Pa(A)}` embedded as the identity on the unused halves,
      supports encoding the induced DAG
- [x] 2.2 Add `CircuitModel::glue` along a shared wire, and its dilation as the induced
      factorization of the composite
- [x] 2.3 Add `CircuitSubject` and `.over_circuit` to `QclBuilder`; `build()` rejects a cyclic
      induced DAG as `CyclicStructureUnsupported`; `Screened<R>` records its origin
- [x] 2.4 Add `NoCompositionalModel` and return it from every abstraction constructor handed a
      model subject
- [x] 2.5 Verify: the dilation of a two-node unitary circuit is Markov for its induced DAG at
      Q-TOL with provenance `Rederived`; the glued dilation produces no `CertificateNotInherited`;
      `.over_model` behaviour unchanged against the existing pipeline tests

## 3. The abstraction object and the naturality check

- [x] 3.1 Add `TypeAlignment` with `τ_X` as a `QcMorphism`, the section `E_X`, the `τ ∘ E = id` check
      against `Tolerance::state()`, `SectionNotInverse`, and monoidal products of types
- [x] 3.2 Add `QuerySignature` with `Io`, `Open(S)`, `Inc(S₁…Sₙ)` and `Observe(O)`; the
      parallelisable check at construction with `NotParallelisable` naming the path; `Open` given
      semantics as box deletion and shown equal to `intervene_mechanism` with the identity
      instrument on the dilation
- [x] 3.3 Add `Abstraction<L, H>` with the total query map, and the derived upward abstraction per
      Proposition 18 computed on demand
- [x] 3.4 Add `check_naturality` as a `Check<R>`: one record per query, the `SemanticsPath`, the
      norm, the amplification factor and the examined count beside the report; the ε-abstraction
      definition in the doc block with the paper cited for the exact case
- [x] 3.5 Write the two-sided bound `r / d_in ≤ diamond ≤ √(d_in d_out) · r` and its derivation
      (`notes/open-questions-resolved.md` §2) into the docstring and the report, and test both ends
      against the `2 sin(θ/2)` closed form over a `θ` sweep, expecting `r = 2√2 sin(θ/2)`
- [x] 3.6 Verify: vacuous signature reads `Vacuous`; a swapped program rejects and names the query;
      both paths agree on the small fixture; defect audit and `cargo mutants` on the residual kernel

## 4. The structural precheck and the code as an abstraction

- [x] 4.1 Add `check_alignment_structure`: `α(X)` by blocked reachability on the low-level DAG, the
      simple, extra-simple and full predicates, the offending pair as witness, and the scope field
      `Equivalent | Necessary` with the doc stating Theorem 51's classical scope and Remark 56's
      argument for the quantum case
- [x] 4.2 Add the paper's Examples 54 and 55 as fixtures and pin their predicate values
- [x] 4.3 Write the generation regression first: `check_naturality` on the strict
      `CodeAbstraction` against `check_class_invariance` and `check_clifford_action` on
      `[[18,2,3]]` and `[[32,2,4]]`, every emitted gate, verdict and witness
- [x] 4.4 Add `CodeAbstraction` from a `LogicalBasis`: `π` to the block, `τ` the ideal decoder from
      the stabilizer generators, `E` the code-space isometry, the query map through the Table 1
      emitters, `Open(S̄) ↦ Open(π(S̄))`; `Observe(Ō)` to the logical measurement deferred (D16)
- [x] 4.5 Wire `check_alignment_structure` and `check_naturality` into `Validate` on the circuit
      subject, in that order, sticky failure, named stages
- [x] 4.6 Verify: the regression passes on both fixtures; omitting `S̄`'s CZ pairs fails it; the
      precheck on a `CircuitModel` reads `Necessary`; a rejecting precheck stops `check_naturality`
      from forming a matrix

## 5. Fault sets and the fault-tolerance predicate

- [x] 5.1 Add `FaultSet` with `pauli_weight(t)`, `declared` and `from_dem`, counted on
      `NumberType` as `C(n, t) · 3^t` and refused above the cap before allocating
- [x] 5.2 Add `GaugeFieldGate` (blocks as `Gf2Chain`s, phase function on `{0,1}^m` as `Turns`),
      constructors for every Table 1 diagonal gate from Eq. (3.63), and the propagator: Clifford
      layers through `clifford_conjugate`, diagonal gates by parity flips through `Gf2Chain::inner`
      with the remainder as a `GaugeFieldGate`, and `NoPropagationNormalForm` for a program with two
      non-Clifford layers separated by a non-diagonal Clifford
- [x] 5.3 Add the exact decision: a fault is tolerated iff the remainder's phase function is
      constant on `{0,1}^m`, with the flipped parity pattern and the phase table as witness; on the
      numeric path, report the remainder's Pauli coefficients
- [x] 5.4 Add `check_fault_tolerance` over the enlarged signature: per-fault residuals, the worst,
      the count, the witness `(location, Pauli, term)`, the `SemanticsPath` per record
- [x] 5.5 Add the Haruna filter with every verdict labelled `Exact`, its expected values carrying
      the Eq. (3.63) derivation of `notes/open-questions-resolved.md` §3 as provenance
- [x] 5.6 Verify: `X_q` through `S̄(γ)` gives `X_q Z̄(γ)` up to phase; `X_q` through `T̄(γ)` gives the
      remainder `exp(±iπ/4 Z̄(γ))` with two terms of modulus `1/√2` at `w = 3, 4, 5`; `X_q X_r`
      through `T̄` does not spread; `Z_q` passes through unchanged; the filter matches the derivation
      on both torus fixtures; the two-layer program is refused by name; the empty set reads
      `Vacuous`; defect audit and `cargo mutants` on the propagator

## 6. Composition and the three chain consumers

- [ ] 6.1 Add `Abstraction::compose`: `π = π₁ ∘ π₂`, `τ = τ₂ ∘ τ₁`, both constants as the
      Frobenius-induced norms of `τ₂` and `τ₁` from the Gram matrix of each natural representation
      through `eigen_hermitian`, the norm and the bound in the report's provenance
- [ ] 6.2 Add `lean/DeepCausalityFormal/Quantum/Abstraction.lean` with Proposition 17 in the exact
      case over the pair-indexed matrix model, and bind it in `lean/THEOREM_MAP.md` to the exact
      composition test; register the Bazel `lean_test` target
- [ ] 6.3 Add the three chain consumers under `examples/quantum_examples/qcl_examples/`:
      the concatenated hand-built `[[4,2,2]]`, code switching with the gadget as low-level query, and a distillation
      round labelled as an example; each with a `rust_binary` in `BUILD.bazel`, a `FloatType` alias
      in `main.rs`, and the lifts from `deep_causality_num`
- [ ] 6.4 Verify: exact links compose to residual zero; the tightness pair exceeds a bound with
      either constant set to one; each consumer's measured residual is at most its recorded bound;
      the consumers run at `f32`, `f64` and `Float106`

## 7. The decoder as an abstraction

- [ ] 7.1 Add `DemModel::from_graph` over a frozen `CausaloidGraph` under `qcm`, with detectors,
      observables and latent mechanisms, and `induced_dag()`
- [ ] 7.2 Add the `dem` feature implying `qcm`, `DemModel::from_stim_text` for `error`, `detector`
      and `logical_observable` lines, unknown lines refused by name; enable `dem` in `BUILD.bazel`
- [ ] 7.3 Add `DecoderAbstraction` with `τ` as a caller-supplied channel or stochastic matrix lifted
      through the FStoch embedding; no `Decoder` trait
- [ ] 7.4 Add the logical attribution query over a `FaultSet`, ranked by residual
- [ ] 7.5 Build the small memory-experiment fixture with one injected correlated two-qubit error
      and its two `DemModel`s, with and without the mechanism
- [ ] 7.6 Verify: the omitted mechanism is exposed at the injected location; the complete model
      passes; attribution ranks the injected location first; the three-line Stim text parses and the
      `repeat` line is refused

## 8. The crosstalk consumer over circuits, and close-out

- [ ] 8.1 Re-express the crosstalk consumer's candidates as `CircuitModel` values with normalised
      dilations and the same parental structure, run `.over_circuit`, and keep the v1 example beside
      it
- [ ] 8.2 Verify: three admitted, the cyclic fourth refused at `build()`, the plan `{do(Q1),
      do(Q2)}` at cost 2 against tomography at 200, H₁ the survivor
- [ ] 8.3 Register every new test file in its `mod.rs` and in `tests/BUILD.bazel`; add every new
      example's `rust_binary`; `make check_examples` green
- [ ] 8.4 Update `qcl-design-note.md` §9 with a QCL-2 row per group, each check's witness through
      `lean/THEOREM_MAP.md` or the statement that it has none, and `LEAN_QUANTUM.md` with the new
      Lean file
- [ ] 8.5 Verify: `bazel test //...` green, `cargo clippy --workspace --all-targets` clean,
      `cargo fmt --check` clean, `openspec validate --specs` green, the default and `no-std` builds
      of `deep_causality_quantum` compile the ungated abstraction layer
