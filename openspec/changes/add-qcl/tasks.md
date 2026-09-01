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

## 2. The decision form

- [ ] 2.1 Add `Tolerance<R>` as a family with named members, each derived from `R::epsilon()`,
      generalising the four shipped policies; add no integer member
- [ ] 2.2 Add `Check<R>` and `CheckReport<R>` carrying `(measured, threshold, margin, verdict)` plus
      the count of items examined, generalising `CommutatorCheck` and `QuantumMarkovReport`
- [ ] 2.3 Add report-returning siblings for `check_completely_positive` and `check_trace_preserving`,
      surfacing the spectrum and the defect they already compute; leave the existing signatures
- [ ] 2.4 Add a report-returning sibling for `quantum_markov_check` that carries the whole report on
      the failure path rather than dropping it at the first failing pair
- [ ] 2.5 Verify: a factorization with disjoint supports reports zero pairs tested and reads as a
      vacuous pass; a rejected candidate still reports its margins and its count

## 3. Carriers

- [ ] 3.1 Add `QubitOperator` with named constructors, replacing hand-packed 2×2 tensors
- [ ] 3.2 Add `Channel`, CPTP-checked once at construction through the shipped checks
- [ ] 3.3 Add `QuantumPlant` as a sealed validated state that evolves in place
- [ ] 3.4 Add `Observable` as a named projector carrying its own Born read-out
- [ ] 3.5 Give each carrier a `wrappers.rs`-style lift into the causal monad
- [ ] 3.6 Verify: no carrier exposes `&mut` to its validated interior, and each rejects a malformed
      construction with a typed `QuantumError` rather than panicking

## 4. Evidence and the shot budget

- [ ] 4.1 Add `ShotBudget` and `Evidence`, with shot statistics generic in the scalar rather than
      pinned to `f64`
- [ ] 4.2 Add the sampler that turns `born_projective_probability` into shots on the `DensityMatrix`
      carrier, in the default build
- [ ] 4.3 Make a config naming a shot budget a compile-time selection of the emergent modality
- [ ] 4.4 Verify: the default build compiles without the `QpuSampler` seam; a run at two scalars
      shows the tolerances moving with the scalar

## 5. Hypothesis and intervention

- [ ] 5.1 Add `Hypothesis` as `{ name, ProcessFactors<R>, FactorSupports }` with its
      `CausalStructure` derived rather than stored
- [ ] 5.2 Add `intervene(do(node ← factor))` as a keyed replacement followed by
      `FactorSupports::validate`
- [ ] 5.3 Add `predict` as model evaluation, contracting the factor network through `space_map` and
      `embed_on_legs`
- [ ] 5.4 Gate every marginalisation on `partial_trace_preservation_boundary` and carry the returned
      `BoundaryWarrant` bound forward
- [ ] 5.5 Verify: a `Screened<R>` whose factorization is marginalised carries an invalidated report
      rather than a stale one

## 6. Design and adjudication

- [ ] 6.1 Add `DesignPlan` carrying the ordered experiment set, total cost, the pair each resolves,
      and any pairs left uncovered
- [ ] 6.2 Add `design` under `MinCostCover`, solved as a DP over covered-pair subsets
- [ ] 6.3 Add `adjudicate` with the §4 fold rule: projection-valued folds check commutation through
      `Projection::commutes_with`, read-outs against a real-valued spec do not
- [ ] 6.4 Return the adjudication outcome as `Either`, one surviving hypothesis against a residual
      ambiguity
- [ ] 6.5 Verify: the crosstalk case selects `{E1, E2}` at cost 2 over tomography at 200; a
      non-commuting projection fold yields `Ambiguous`; a threshold fold is not rejected by the guard

## 7. The pipeline

- [ ] 7.1 Add `Ledger<R, N>` with counts on `NaturalNumber`, `checked_difference` for the draw-down,
      and its three invariants
- [ ] 7.2 Add `fork` above core by cloning, one live world per hypothesis with its own ledger
- [ ] 7.3 Add `QclBuilder::config::<FloatType, IntType>()` and the three subject constructors
- [ ] 7.4 Enforce the `build()` preconditions, including rejecting an unfrozen graph
- [ ] 7.5 Add the stages, with `validate` terminating in `Screened<R>` and `control` accepting either
      a plant config or a `Screened<R>`
- [ ] 7.6 Make failure transactional, rolling back and carrying the structured `QuantumError`
- [ ] 7.7 Verify: a structural config has no path into `control` that skips `validate`; a failed
      `validate` leaves nothing half-frozen

## 8. Consumers and close-out

- [ ] 8.1 Express the QCM path against the shipped `freeze_quantum` callers
- [ ] 8.2 Express the geometric-QEC path, running `derive_code`, `check_ldpc_weights` and
      `check_class_invariance`
- [ ] 8.3 Express the crosstalk path, the keystone that exercises the `validate` to `control`
      hand-off
- [ ] 8.4 Add a `rust_binary` in `BUILD.bazel` for every new Cargo example, and register every new
      test file in its `mod.rs` and in `tests/BUILD.bazel`
- [ ] 8.5 Verify: `bazel test //...` green, `cargo clippy --workspace --all-targets` clean,
      `cargo fmt --check` clean, `openspec validate --specs` green
- [ ] 8.6 Update `qcl-design-note.md` §9 to record what shipped, and name each check's Rust witness
      through `lean/THEOREM_MAP.md` or say it has no proof
