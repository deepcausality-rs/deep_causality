<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# LEAN_QUANTUM — verification status of `deep_causality_quantum`

This crate reconstructs a quantum causal model (QCM; Lorenz 2022, Lorenz & Barrett 2021) on the
DeepCausality causal monad. Its claims are split by **modality**, and the two modalities carry
different kinds of evidence.

## The two modalities

- **Verifiable (default build).** Deterministic simulated Choi–Jamiołkowski operators carried as an
  external freeze-time decoration; the quantum Markov condition recovered as a freeze-boundary
  commutativity check. This is the path the Lean proofs attach to.
- **Emergent (`qpu` feature, off by default).** A physical cloud-QPU call as a monadic effect —
  the `QpuSampler` seam, the in-process `SimQpu`, and the `qpu_effect` lift. Not a Lean target by
  nature: its evidence is tests and provenance, and a concrete vendor adapter is out of scope.

The default `cargo` build compiles only the verifiable path and pulls in no network/async
dependency; the modality separation is a compile-time guarantee.

## Lean formalization (Lean v4.32.0 + Mathlib)

The pinned Mathlib has neither a partial trace nor a Choi–Jamiołkowski layer, so both are built from
first principles in `lean/DeepCausalityFormal/Quantum/`. Every listed theorem closes with **zero
`sorry`**; each is bound to a Rust witness through `lean/THEOREM_MAP.md`.

| Lean theorem | statement | Rust witness |
|---|---|---|
| `partialTraceRight_add` / `_smul` | partial trace is linear | `operator_linalg_tests :: test_partial_trace_linearity` |
| `partialTraceRight_kron` | `Tr_B(X⊗Y) = Tr(Y)•X` | `… :: test_partial_trace_product_identity` |
| `partialTraceRight_bimodule` / `_right` | `Tr_B((Z⊗1)·M) = Z·Tr_B(M)` (both sides) | `… :: test_partial_trace_bimodule_law` |
| `partial_trace_preservation_boundary` | boundary support ⇒ commutation preserved (Q-PTP) | `… :: test_partial_trace_preservation_boundary_case` |
| `partial_trace_nonpreservation` (+ `_value`) | **B1**: `[X,Y]=0` yet `[Tr_B X, Tr_B Y] = [[0,4],[−4,0]] ≠ 0` | `… :: test_partial_trace_nonpreservation_counterexample` |
| `applyChoi_add` / `applyChoi_smul` | the reconstructed Choi action is linear | `channel_tests :: test_apply_kraus_and_apply_choi_agree` |
| `abstraction_compose_exact` / `_defect` | two commuting abstraction squares paste along their middle level (Lorenz & Tull, Proposition 17, exact case) | `composition_tests :: test_exact_links_compose_exactly_on_the_concatenated_code` |
| `applyChoi_choiOf` | the Choi–Jamiołkowski reconstruction: `applyChoi (choiOf E) A = E A` for every linear channel | `choi_tests :: test_choi_reconstruction_recovers_the_channel` |
| `subspace_lattice_orthomodular` / `subspace_lattice_not_distributive` | the subspace lattice behind the `Projection` verdict is orthomodular and not distributive | `orthomodular_tests :: test_the_orthomodular_law_holds_for_a_line_inside_a_plane` |
| `kron_one_commute_one_kron` / `pairwise_commute_prod_perm` | factors on disjoint legs commute; a pairwise-commuting product does not depend on its order (Lorenz 2022, Def. 3.3) | `qcm_theorem_tests :: test_factors_on_disjoint_legs_commute` |
| `diagonal_commute` / `kron_diagonal` | diagonal (classical) factors commute, and their Kronecker product is diagonal | `qcm_theorem_tests :: test_diagonal_factors_commute_and_their_kronecker_product_is_diagonal` |
| `noInfluence_iff` | the D marginal factors through `Tr_A` exactly when it depends on the A input only through its trace (Lorenz & Barrett 2021, Def. 1) | `qcm_theorem_tests :: test_a_product_unitary_has_no_influence_from_a_to_d` |

### The headline: `partial_trace_preservation` is false

The roadmap's `quantum.partial_trace_preservation` is **not true unconditionally** — partial trace
is positive-linear but not an algebra homomorphism. The refuting witness is proved in Lean (closed by
`decide` over `ℤ`, since the physics writeup's `+4i·σy` is the integer matrix `[[0,4],[−4,0]]`). What
*is* true is the **conditional** `partial_trace_preservation_boundary`: a boundary operator `Z ⊗ 1_B`
commuting with `M` forces `Z` to commute with `Tr_B(M)`. The crate therefore supports **flat** QCM
models and treats quantum-subgraph nesting — whose physical meaning is itself unestablished — as an
open research question, not a promised feature.

### The abstraction layer (QCL-2)

`lean/DeepCausalityFormal/Quantum/Abstraction.lean` states Proposition 17 in the exact case over
the pair-indexed matrix model: with the two link squares `τ₁out · L = M · τ₁in` and
`τ₂out · M = H · τ₂in`, the composite `(τ₂out · τ₁out) · L = H · (τ₂in · τ₁in)`, which is associativity
of matrix multiplication applied twice, and the same in defect form. The approximate law the crate
records, `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` in Frobenius norm on Choi operators with both constants
computed, is the crate's own theorem by the triangle inequality on the same pasting and has no Lean
statement. The naturality check, the structural precheck, the fault propagator and the decoder
validation are computed and witnessed by tests; none claims a theorem.

## Open target

One target is stated and not proved: `quantum.unitary_factorization` (Lorenz & Barrett 2021,
Theorem 1), the commuting factorization on the unitary fragment. Its proof rests on the commutant
and direct-sum decomposition of finite-dimensional C\*-algebras. The pinned Mathlib has
Wedderburn–Artin and C\*-algebra basics but not that decomposition. The
`lean/DeepCausalityFormal/Quantum/` tree is exempt from the CI `sorry` gate while this foundation is
extended.

`quantum.cyclic_support` is not a target. The crate refuses cyclic structures at `build()` by
decision, and acyclicity as a separable parameter is proved as
`core.context_graph.acyclicity_separable`.

## Faithfulness scope

Faithfulness claims are limited to the **C₃-exclusion (traditional-circuit)** regime of van der Lugt
& Lorenz (arXiv:2508.11762): a declared causal structure with no `C₃` sub-relation is faithfully
representable, and a `C₃`-containing structure is rejected at freeze. The general routed/direct-sum
Lorenz–Barrett hypothesis remains open upstream and is out of scope.
