/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — the partial-trace non-preservation counterexample (the B1 witness).

`quantum.partial_trace_preservation` is **false unconditionally**: partial trace is positive-linear
but not an algebra homomorphism, so it has no general reason to send commutators to commutators. The
refuting witness (Lorenz–Barrett-flavoured, finite, 2-qubit): with `X = σx ⊗ |0⟩⟨0| + σz ⊗ |1⟩⟨1|`
and `Y = σx ⊗ |0⟩⟨0| − σz ⊗ |1⟩⟨1|`, `[X, Y] = 0` yet `[Tr_B X, Tr_B Y] = [[0,4],[−4,0]] ≠ 0`.

Every entry here is an **integer** (the imaginary `+4i·σy` of the physics writeup is `[[0,4],[−4,0]]`
as a matrix), so the witness is stated over `ℤ` and closed by `decide` — no `sorry`, no complex
arithmetic. This is the exact counterpart of the crate's numerical witness
`test_partial_trace_nonpreservation_counterexample`.

Rust witness: `deep_causality_quantum/tests/kernels/operator_linalg_tests.rs ::
test_partial_trace_nonpreservation_counterexample`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

import DeepCausalityFormal.Quantum.PartialTrace

-- The pair-indexed matrix model leaves some section instances unused per-lemma; this is
-- the standard Mathlib idiom for that situation and keeps the proofs uncluttered.
set_option linter.unusedSectionVars false

namespace DeepCausalityFormal.Quantum

open Matrix

/-- Pauli-X over `ℤ`. -/
def σx : Matrix (Fin 2) (Fin 2) ℤ := !![0, 1; 1, 0]
/-- Pauli-Z over `ℤ`. -/
def σz : Matrix (Fin 2) (Fin 2) ℤ := !![1, 0; 0, -1]
/-- `|0⟩⟨0|`. -/
def e00 : Matrix (Fin 2) (Fin 2) ℤ := !![1, 0; 0, 0]
/-- `|1⟩⟨1|`. -/
def e11 : Matrix (Fin 2) (Fin 2) ℤ := !![0, 0; 0, 1]

/-- `X = σx ⊗ |0⟩⟨0| + σz ⊗ |1⟩⟨1|`. -/
def Xctr : Matrix (Fin 2 × Fin 2) (Fin 2 × Fin 2) ℤ := kron σx e00 + kron σz e11
/-- `Y = σx ⊗ |0⟩⟨0| − σz ⊗ |1⟩⟨1|`. -/
def Yctr : Matrix (Fin 2 × Fin 2) (Fin 2 × Fin 2) ℤ := kron σx e00 - kron σz e11

/-- **Non-preservation of commutation under partial trace (B1).** The two operators commute, yet
    their partial traces do not — the refuting witness for the unconditional
    `partial_trace_preservation`. Closed by `decide` over `ℤ`.

    THEOREM_MAP: `quantum.partial_trace_nonpreservation` -/
theorem partial_trace_nonpreservation :
    Xctr * Yctr = Yctr * Xctr ∧
    partialTraceRight Xctr * partialTraceRight Yctr
      ≠ partialTraceRight Yctr * partialTraceRight Xctr := by
  refine ⟨?_, ?_⟩
  · decide
  · decide

/-- The commutator of the partial traces is exactly `[[0, 4], [−4, 0]]` — the integer form of
    `+4i·σy`.

    THEOREM_MAP: `quantum.partial_trace_nonpreservation.value` -/
theorem partial_trace_nonpreservation_value :
    partialTraceRight Xctr * partialTraceRight Yctr
      - partialTraceRight Yctr * partialTraceRight Xctr = !![0, 4; -4, 0] := by
  decide

end DeepCausalityFormal.Quantum
