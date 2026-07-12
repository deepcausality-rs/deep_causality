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

## Lean formalization (Lean 4.15.0 + Mathlib)

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

### The headline: `partial_trace_preservation` is false

The roadmap's `quantum.partial_trace_preservation` is **not true unconditionally** — partial trace
is positive-linear but not an algebra homomorphism. The refuting witness is proved in Lean (closed by
`decide` over `ℤ`, since the physics writeup's `+4i·σy` is the integer matrix `[[0,4],[−4,0]]`). What
*is* true is the **conditional** `partial_trace_preservation_boundary`: a boundary operator `Z ⊗ 1_B`
commuting with `M` forces `Z` to commute with `Tr_B(M)`. The crate therefore supports **flat** QCM
models and treats quantum-subgraph nesting — whose physical meaning is itself unestablished — as an
open research question, not a promised feature.

## Deferred (stated as targets, not yet proved in Lean)

These carry numerical / property-test witnesses in the crate today; their Lean proofs need net-new
Mathlib machinery and are future work. The `lean/DeepCausalityFormal/Quantum/` tree is exempt from
the CI `sorry` gate while this foundation is extended.

- The **Choi–Jamiołkowski reconstruction isomorphism** `applyChoi (choiOf E) = E` (round-trip
  witnessed by `channel_tests`).
- `quantum.no_influence`, `quantum.markov_commutativity` (the freeze check is witnessed by
  `qcm/markov_freeze_tests`).
- `quantum.unitary_factorization` (research-grade; needs the direct-sum / C\*-structure theory
  Mathlib lacks).
- `quantum.classical_embedding`, `quantum.cyclic_support`.
- `quantum.verdict.orthomodular` — the Rust orthomodular projection-lattice `Verdict` carrier and its
  law tests are complete (`verdict/projection_tests`); the Lean statement extending
  `core.verdict.carriers` is future work.

## Faithfulness scope

Faithfulness claims are limited to the **C₃-exclusion (traditional-circuit)** regime of van der Lugt
& Lorenz (arXiv:2508.11762): a declared causal structure with no `C₃` sub-relation is faithfully
representable, and a `C₃`-containing structure is rejected at freeze. The general routed/direct-sum
Lorenz–Barrett hypothesis remains open upstream and is out of scope.
