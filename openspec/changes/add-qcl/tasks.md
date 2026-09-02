<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Ordering follows `qcl-design-note.md` §9, with group 1 ahead of it because that work is already in
the working tree and half finished. Every group ends with a verification task, and no group is done
until `bazel test //...` is green for it.

## 1. Finish `check_class_invariance` over the code space

- [x] 1.1 Derive the Z-stabilizer generators in `LogicalBasis::from_complex` as a basis of `im ∂ₖ₊₁`
      and expose them; no second constructor, and no X-stabilizers, which a diagonal phase cannot see
- [x] 1.2 Replace the commutation criterion with triviality of the ratio on the code space, imposing
      `|b ∩ x|` even only where the shift lies in the stabilizer span
- [x] 1.3 Keep the explicit enumeration cap and its typed error, and report the shifts examined and
      the states visited so an expensive code fails loudly rather than hanging
- [x] 1.4 Verify: `test_every_gate_acts_on_the_class_not_the_representative` passes for Z̄, S̄ and T̄
      on the 3×3 torus, and the Z̄ path still agrees with the shipped Pauli predicate
- [x] 1.5 Verify by mutation: dropping the code-space restriction, applying it unconditionally,
      flipping the integrality test to rational equality, and reading the wrong overlap each fail at
      least one test
- [x] 1.6 Derive the X-stabilizer generators in `from_complex` as a basis of `im δₖ₋₁`, and make
      `is_logically_trivial` check the normalizer precondition against both generator sets,
      returning `NotInNormalizer` with the offending generator (X-6)
- [x] 1.7 Carry `logical_hadamard`'s global phase on the emitted program as `global_phase`, populated
      by the causal wrapper rather than dropped (X-11)
- [x] 1.8 Add `check_clifford_action`: a symplectic tableau update over `GateOp` for `H`, `S` and
      `CZ`, refusing non-Clifford gates, deciding `Z̄(γ) ↔ X̄(γ̃)` through `LogicalBasis` up to phase
      (X-5)
- [x] 1.9 Make `logical_t` and `logical_multi_cz` report their tuple count and error above a
      configurable cap before allocating (X-13)
- [x] 1.10 Verify: `H̄` on the 3×3 torus swaps the logical Paulis; `S̄` and `CZ̄` agree between the
      two stages; a `T` program is refused by the Clifford stage; an `X` on a stabilized qubit is
      `NotInNormalizer`

## 2. The decision form

- [x] 2.1 Add `Tolerance<R>` as a family with named members, each derived from `R::epsilon()`,
      generalising the four shipped policies; add no integer member
- [x] 2.2 Add `Check<R>` and `CheckReport<R>` carrying `(measured, threshold, margin, verdict)` plus
      the count of items examined, generalising `CommutatorCheck` and `QuantumMarkovReport`
- [x] 2.3 Add report-returning siblings for `check_completely_positive` and `check_trace_preserving`,
      surfacing the spectrum and the defect they already compute; leave the existing signatures
- [x] 2.4 Add a report-returning sibling for `quantum_markov_check` that carries the whole report on
      the failure path rather than dropping it at the first failing pair
- [x] 2.5 Verify: a factorization with disjoint supports reports zero pairs tested and reads as a
      vacuous pass; a rejected candidate still reports its margins and its count
- [x] 2.6 Add the `Inherited | Rederived` provenance to the Markov `CheckReport<R>` and the
      `CertificateNotInherited` error variant, with the failure variant following the provenance
      (X-2)

## 3. Carriers

- [x] 3.1 Add `QubitOperator` with named constructors, replacing hand-packed 2×2 tensors
- [x] 3.2 Add `Channel`, CPTP-checked once at construction through the shipped checks
- [x] 3.3 Add `QuantumPlant` as a sealed validated state that evolves in place
- [x] 3.4 Add `Observable` as a named projector carrying its own Born read-out
- [x] 3.5 Give each carrier a `wrappers.rs`-style lift into the causal monad
- [x] 3.6 Verify: no carrier exposes `&mut` to its validated interior, and each rejects a malformed
      construction with a typed `QuantumError` rather than panicking

## 4. Evidence and the shot budget

- [x] 4.1 Add `ShotBudget` and `Evidence`, with shot statistics generic in the scalar rather than
      pinned to `f64`
- [x] 4.2 Add the sampler that turns `born_projective_probability` into shots on the `DensityMatrix`
      carrier, in the default build
- [x] 4.3 Make a config naming a shot budget a compile-time selection of the emergent modality
- [x] 4.4 Verify: the default build compiles without the `QpuSampler` seam; a run at two scalars
      shows the tolerances moving with the scalar

## 5. Hypothesis and intervention

- [x] 5.0 Correct `is_c3_block` to Definition 3.1 — sorted degrees `[2, 2, 3]` on rows and columns —
      rewrite the module docs against Example 2.12, Theorem 3.2 and Remark 3.3, and hold the tests
      to the paper and to Theorem 4.9(v) over all 512 small relations (X-16)
- [x] 5.1 Add `Hypothesis` as `{ name, ProcessFactors<R>, FactorSupports }` with its
      `CausalStructure` derived rather than stored
- [x] 5.2 Add `intervene_mechanism(do(node ← factor))` as a keyed replacement followed by
      `FactorSupports::validate`, documented as the mechanism-level intervention with
      `intervene_instrument` reserved and not built (X-4)
- [x] 5.3 Add `predict` as model evaluation, contracting the factor network through `space_map` and
      `embed_on_legs`
- [x] 5.4 Gate every marginalisation on `partial_trace_preservation_boundary` and carry the returned
      `BoundaryWarrant` bound forward
- [x] 5.5 Verify: a `Screened<R>` whose factorization is marginalised carries an invalidated report
      rather than a stale one — verified on `Hypothesis`, which drops its certificate on
      `intervene_mechanism` and on `marginalise` and reports the prior margin degraded by `√(d_B)`;
      `Screened<R>` in group 7 wraps a `Hypothesis` and inherits the behaviour

## 6. Design and adjudication

- [x] 6.1 Add `DesignPlan` carrying the ordered experiment set, total cost, the pair each resolves,
      and any pairs left uncovered
- [x] 6.2 Add `design` under `MinCostCover`, solved as a DP over covered-pair subsets, with
      `max_hypotheses` defaulting to 7 and `HypothesisCountExceeded { n, pairs }` above it (X-12)
- [x] 6.3 Add `adjudicate` with the §4 fold rule: projection-valued folds check commutation through
      `Projection::commutes_with`, read-outs against a real-valued spec do not
- [x] 6.4 Return the adjudication outcome as `Either`, one surviving hypothesis against a residual
      ambiguity
- [x] 6.5 Verify: the crosstalk case selects `{E1, E2}` at cost 2 over tomography at 200; a
      non-commuting projection fold yields `Ambiguous`; a threshold fold is not rejected by the guard

## 7. The pipeline

- [ ] 7.1 Add `Ledger<R, N>` with counts on `NaturalNumber`, `checked_difference` for the draw-down,
      and its three invariants
- [ ] 7.2 Add `fork` above core by cloning, one live world per hypothesis with its own ledger
- [ ] 7.3 Add `QclBuilder::config::<FloatType, NumberType>()` and the three subject constructors
- [ ] 7.4 Enforce the `build()` preconditions, including rejecting an unfrozen graph and rejecting
      a cyclic structural candidate as `CyclicStructureUnsupported` before any check runs (X-3)
- [ ] 7.5 Add the stages, named `check_markov`, `check_decomposable`, `check_ldpc_weights`,
      `check_class_invariance` and `check_clifford_action`, with `validate` terminating in
      `Screened<R>` and `control` accepting either a plant config or a `Screened<R>` (X-3)
- [ ] 7.6 Make failure transactional, rolling back and carrying the structured `QuantumError`
- [ ] 7.7 Verify: a structural config has no path into `control` that skips `validate`; a failed
      `validate` leaves nothing half-frozen

## 8. Consumers and close-out

- [ ] 8.1 Express the QCM path against the shipped `freeze_quantum` callers
- [ ] 8.2 Express the geometric-QEC path, running `derive_code`, `check_ldpc_weights`,
      `check_class_invariance` and `check_clifford_action`, and stating that it is verified by exact
      predicates and not simulated (X-7)
- [ ] 8.3 Express the crosstalk path, the keystone that exercises the `validate` to `control`
      hand-off, with the cyclic H₄ rejected at `build()` and the `.adjudicate()` fold documented as
      Boolean (X-3, X-10)
- [ ] 8.4 Add a `rust_binary` in `BUILD.bazel` for every new Cargo example, and register every new
      test file in its `mod.rs` and in `tests/BUILD.bazel`
- [ ] 8.5 Verify: `bazel test //...` green, `cargo clippy --workspace --all-targets` clean,
      `cargo fmt --check` clean, `openspec validate --specs` green
- [ ] 8.6 Update `qcl-design-note.md` §9 to record what shipped, and name each check's Rust witness
      through `lean/THEOREM_MAP.md` or say it has no proof
