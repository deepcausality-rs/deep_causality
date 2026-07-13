---
title: Quantum
description: Quantum partial-trace, Choi, and channel laws built from first principles in Lean, including the partial-trace preservation counterexample, bound to Rust witnesses.
sidebar:
  order: 7
---

Ten laws for the partial-trace / Choi foundation: partial-trace linearity, product, and bimodule laws, the B1 preservation result, and the Choi application laws — built from first principles on the pair-indexed matrix model, because the pinned Mathlib carries neither partial trace nor a Choi–Jamiołkowski layer. Proved in [`lean/DeepCausalityFormal/Quantum/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal/Quantum) and checked by law-tests in `deep_causality_quantum/tests/formalization_lean/`.

The headline is a proved impossibility, not an ordinary law: the unconditional `partial_trace_preservation` is **false** — `partial_trace_nonpreservation` is a witnessed counterexample, with its commutator value pinned exactly — while the *conditional* boundary version holds (`partial_trace_preservation_boundary`). The `/Quantum/` tree is exempt from the `sorry` CI gate while this foundation grows.

Every row below is `proved` in Lean. The **Lean proof** cells are relative to `lean/DeepCausalityFormal/`; the **Rust witness** cells give the test file and name inside the witness directory above.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| `quantum.partial_trace.add` | `Tr_B(M+N) = Tr_B M + Tr_B N` | `Quantum/PartialTrace.lean :: partialTraceRight_add` | `partial_trace_tests.rs :: test_partial_trace_linearity` | ✓ |
| `quantum.partial_trace.smul` | `Tr_B(c•M) = c•Tr_B M` | `Quantum/PartialTrace.lean :: partialTraceRight_smul` | `partial_trace_tests.rs :: test_partial_trace_linearity` | ✓ |
| `quantum.partial_trace.kronecker` | `Tr_B(X⊗Y) = Tr(Y)•X` | `Quantum/PartialTrace.lean :: partialTraceRight_kron` | `partial_trace_tests.rs :: test_partial_trace_product_identity` | ✓ |
| `quantum.partial_trace.bimodule` | `Tr_B((Z⊗1)·M) = Z·Tr_B M` | `Quantum/PartialTrace.lean :: partialTraceRight_bimodule` | `partial_trace_tests.rs :: test_partial_trace_bimodule_law` | ✓ |
| `quantum.partial_trace.bimodule_right` | `Tr_B(M·(Z⊗1)) = Tr_B M·Z` | `Quantum/PartialTrace.lean :: partialTraceRight_bimodule_right` | `partial_trace_tests.rs :: test_partial_trace_bimodule_law` | ✓ |
| `quantum.partial_trace_preservation_boundary` | boundary op commutes ⇒ its A-part commutes with `Tr_B M` (Q-PTP) | `Quantum/PartialTrace.lean :: partial_trace_preservation_boundary` | `partial_trace_tests.rs :: test_partial_trace_preservation_boundary_case` | ✓ |
| `quantum.partial_trace_nonpreservation` | `[X,Y]=0` but `[Tr_B X, Tr_B Y] ≠ 0` (B1 counterexample) | `Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation` | `partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample` | ✓ |
| `quantum.partial_trace_nonpreservation.value` | `[Tr_B X, Tr_B Y] = [[0,4],[−4,0]]` (`= +4i·σy`) | `Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation_value` | `partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample` | ✓ |
| `quantum.choi.apply_add` | `applyChoi J` is additive in the state | `Quantum/Choi.lean :: applyChoi_add` | `choi_tests.rs :: test_apply_choi_is_linear` | ✓ |
| `quantum.choi.apply_smul` | `applyChoi J (c•A) = c•applyChoi J A` | `Quantum/Choi.lean :: applyChoi_smul` | `choi_tests.rs :: test_apply_choi_is_linear` | ✓ |

The CJ reconstruction isomorphism `applyChoi (choiOf E) = E` and the QCM theorems are stated as deferred targets in the `add-quantum-crate` change; the crate carries their numerical and property-test witnesses today.
