## Why

Seven quantum targets carry tests and no Lean proof, and the documented reason, net-new Mathlib
machinery, holds for only one of them. The pinned Mathlib already provides what the CJ
reconstruction, the orthomodular lattice, Markov commutativity and the classical embedding need, so
the proof page can shrink from seven open targets to one whose difficulty it can state exactly.

## What Changes

- Prove the Choi–Jamiołkowski reconstruction `applyChoi (choiOf E) = E` in `Quantum/Choi.lean`.
- Prove `quantum.verdict.orthomodular` over the subspace lattice of a finite-dimensional complex
  inner product space, with a witness in `ℂ²` that distributivity fails.
- State and prove `quantum.markov_commutativity`: factors on disjoint legs commute, and a
  pairwise-commuting product does not depend on its order.
- State and prove `quantum.classical_embedding`: diagonal factors commute pairwise, so a classical
  model meets the Markov condition.
- State and prove `quantum.no_influence`: a marginal that factors through the partial trace over A
  is exactly one that does not depend on the A input.
- **BREAKING** (spec scope): retire `quantum.cyclic_support`. The crate refuses cyclic structures at
  `build()` by decision, no statement was ever written, and the separability fact it would rest on
  is already proved as `core.context_graph.acyclicity_separable`.
- Keep `quantum.unitary_factorization` as the one open target, with the reason recorded: its proof
  rests on the commutant and direct-sum structure of finite-dimensional C*-algebras, which Mathlib
  lacks.
- Add a Rust witness for each new id that lacks one, and a `THEOREM_MAP.md` row for each proved id.
- Correct the stale facts: the Lean and Mathlib version in `LEAN_QUANTUM.md` and `THEOREM_MAP.md`
  (4.15.0 there, v4.32.0 in `lean-toolchain`), the `stdBasisMatrix` comment in `Choi.lean`, and the
  live spec's reference to `deep_causality_do_calculus`, which is not a workspace crate.
- Update the quantum website and the docs-site formalization page to the new counts and to the one
  remaining target.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `quantum-formalization`: the in-scope theorem list changes. `cyclic_support` is retired,
  `unitary_factorization` becomes the named open target, the CJ reconstruction joins the proved
  set, and each remaining id gets a precise statement. The orthomodular requirement states the
  subspace-lattice form and the distributivity witness.
- `quantum-formalization-docs`: the docs-site quantum page lists every proved row and points to the
  single open target instead of the deferred set.

## Impact

- **Lean:** `lean/DeepCausalityFormal/Quantum/` gains proofs in `Choi.lean` and new files for the
  orthomodular lattice, Markov commutativity, the classical embedding and no-influence;
  `lean/DeepCausalityFormal.lean` imports them; `lean/THEOREM_MAP.md` gains rows. The trimmed
  Mathlib olean set may need regenerating if the inner-product-space modules sit outside it.
- **Rust:** new witness tests under `deep_causality_quantum/tests/formalization_lean/` where none
  exists. No library API changes.
- **Docs:** `deep_causality_quantum/LEAN_QUANTUM.md`, `website/docs/src/content/docs/formalization/quantum.md`.
- **Website:** `website/quantum/src/data/formalization.ts`, `src/data/evidence.ts`,
  `src/data/checks.ts`, `src/pages/proof/index.astro`, `src/components/home/Evidence.astro`.
