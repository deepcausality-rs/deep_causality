---
title: Quantum
description: Quantum partial-trace, Choi, and channel laws built from first principles in Lean, including the partial-trace preservation counterexample, bound to Rust witnesses.
sidebar:
  order: 9
---

Nineteen laws for the quantum foundation: partial-trace linearity, product, and bimodule laws, the B1 preservation result, the Choi application laws and the Choi–Jamiołkowski reconstruction, exact abstraction composition, the orthomodular subspace lattice, and the quantum causal-model laws (Markov commutativity, the classical embedding, no influence) — built from first principles on the pair-indexed matrix model, because the pinned Mathlib carries neither partial trace nor a Choi–Jamiołkowski layer. Proved in [`lean/DeepCausalityFormal/Quantum/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal/Quantum) and checked by law-tests in `deep_causality_quantum/tests/formalization_lean/`.

The headline is a proved impossibility, not an ordinary law: the unconditional `partial_trace_preservation` is **false** — `partial_trace_nonpreservation` is a witnessed counterexample, with its commutator value pinned exactly — while the *conditional* boundary version holds (`partial_trace_preservation_boundary`). The `/Quantum/` tree is exempt from the `sorry` CI gate while this foundation grows.

Every row below is `proved` in Lean. The **Lean proof** cells are relative to `lean/DeepCausalityFormal/`; the **Rust witness** cells give the test file and name inside the witness directory above.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| `quantum.partial_trace.add` | `Tr_B(M+N) = Tr_B M + Tr_B N` | `Quantum/PartialTrace.lean :: partialTraceRight_add` | `partial_trace_tests.rs :: test_partial_trace_linearity` | ✓ |
| `quantum.partial_trace.sub` | `Tr_B(M−N) = Tr_B M − Tr_B N` | `Quantum/PartialTrace.lean :: partialTraceRight_sub` | `partial_trace_tests.rs :: test_partial_trace_linearity` | ✓ |
| `quantum.partial_trace.smul` | `Tr_B(c•M) = c•Tr_B M` | `Quantum/PartialTrace.lean :: partialTraceRight_smul` | `partial_trace_tests.rs :: test_partial_trace_linearity` | ✓ |
| `quantum.partial_trace.kronecker` | `Tr_B(X⊗Y) = Tr(Y)•X` | `Quantum/PartialTrace.lean :: partialTraceRight_kron` | `partial_trace_tests.rs :: test_partial_trace_product_identity` | ✓ |
| `quantum.partial_trace.bimodule` | `Tr_B((Z⊗1)·M) = Z·Tr_B M` | `Quantum/PartialTrace.lean :: partialTraceRight_bimodule` | `partial_trace_tests.rs :: test_partial_trace_bimodule_law` | ✓ |
| `quantum.partial_trace.bimodule_right` | `Tr_B(M·(Z⊗1)) = Tr_B M·Z` | `Quantum/PartialTrace.lean :: partialTraceRight_bimodule_right` | `partial_trace_tests.rs :: test_partial_trace_bimodule_law` | ✓ |
| `quantum.partial_trace_preservation_boundary` | boundary op commutes ⇒ its A-part commutes with `Tr_B M` (Q-PTP) | `Quantum/PartialTrace.lean :: partial_trace_preservation_boundary` | `partial_trace_tests.rs :: test_partial_trace_preservation_boundary_case` | ✓ |
| `quantum.partial_trace.commutator_transport` | `Tr_B([Z⊗1,M]) = [Z,Tr_B M]`, unconditional (Q-PTT) | `Quantum/PartialTrace.lean :: partialTraceRight_commutator` | `partial_trace_tests.rs :: test_partial_trace_transports_the_commutator_exactly` | ✓ |
| `quantum.partial_trace_nonpreservation` | `[X,Y]=0` but `[Tr_B X, Tr_B Y] ≠ 0` (B1 counterexample) | `Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation` | `partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample` | ✓ |
| `quantum.partial_trace_nonpreservation.value` | `[Tr_B X, Tr_B Y] = [[0,4],[−4,0]]` (`= +4i·σy`) | `Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation_value` | `partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample` | ✓ |
| `quantum.choi.apply_add` | `applyChoi J` is additive in the state | `Quantum/Choi.lean :: applyChoi_add` | `choi_tests.rs :: test_apply_choi_is_linear` | ✓ |
| `quantum.choi.apply_smul` | `applyChoi J (c•A) = c•applyChoi J A` | `Quantum/Choi.lean :: applyChoi_smul` | `choi_tests.rs :: test_apply_choi_is_linear` | ✓ |
| `quantum.choi.reconstruction` | `applyChoi (choiOf E) A = E A` for every linear channel `E`: the Choi–Jamiołkowski reconstruction | `Quantum/Choi.lean :: applyChoi_choiOf` | `choi_tests.rs :: test_choi_reconstruction_recovers_the_channel` | ✓ |
| `quantum.verdict.orthomodular` | the lattice of subspaces of a finite-dimensional complex inner product space, the ranges of the Rust `Projection` carrier, is bounded, its orthocomplement is an antitone involution with `K ⊓ Kᗮ = ⊥` and `K ⊔ Kᗮ = ⊤`, and it is orthomodular; distributivity fails on the lines through two independent vectors and their sum | `Quantum/Orthomodular.lean :: subspace_lattice_orthomodular / subspace_lattice_not_distributive` | `orthomodular_tests.rs :: test_the_orthomodular_law_holds_for_a_line_inside_a_plane` | ✓ |
| `quantum.markov_commutativity` | factors on disjoint legs commute, `(A⊗1)(1⊗B) = (1⊗B)(A⊗1)`, and a pairwise-commuting product does not depend on its order (Lorenz 2022, Def. 3.3) | `Quantum/Markov.lean :: kron_one_commute_one_kron / pairwise_commute_prod_perm` | `qcm_theorem_tests.rs :: test_factors_on_disjoint_legs_commute` | ✓ |
| `quantum.classical_embedding` | diagonal factors commute pairwise, so a classical model meets the Markov condition, and `diag a ⊗ diag b = diag (a·b)` | `Quantum/ClassicalEmbedding.lean :: diagonal_commute / diagonal_prod_perm / kron_diagonal` | `qcm_theorem_tests.rs :: test_diagonal_factors_commute_and_their_kronecker_product_is_diagonal` | ✓ |
| `quantum.no_influence` | the D marginal factors through `Tr_A` exactly when the D marginal of every product input depends on its A part only through the trace (Lorenz & Barrett 2021, Def. 1, for any linear map) | `Quantum/NoInfluence.lean :: noInfluence_iff` | `qcm_theorem_tests.rs :: test_a_product_unitary_has_no_influence_from_a_to_d` | ✓ |
| `quantum.abstraction.compose_exact` | two commuting abstraction squares paste: `(τ₂τ₁)·L = H·(σ₂σ₁)` (Lorenz & Tull Prop. 17, exact case) | `Quantum/Abstraction.lean :: abstraction_compose_exact` | `composition_tests.rs :: test_exact_links_compose_exactly_on_the_concatenated_code` | ✓ |
| `quantum.abstraction.compose_exact.defect` | zero link defects give a zero composite defect, the form the Rust residual measures | `Quantum/Abstraction.lean :: abstraction_compose_exact_defect` | `composition_tests.rs :: test_exact_links_compose_exactly_on_the_concatenated_code` | ✓ |

One quantum target is open: `quantum.unitary_factorization` (Lorenz & Barrett 2021, Theorem 1), the commuting factorization on the unitary fragment, whose proof needs the commutant and direct-sum decomposition of finite-dimensional C*-algebras that the pinned Mathlib lacks.
