# Closing the deferred quantum Lean targets

## What

Seven quantum targets carry tests and no Lean proof. Six of them close in this change; one stays
open with its reason stated.

| Target | Decision |
|---|---|
| CJ reconstruction, `applyChoi (choiOf E) = E` | prove |
| `quantum.verdict.orthomodular` | prove |
| `quantum.markov_commutativity` | state, then prove |
| `quantum.classical_embedding` | state, then prove |
| `quantum.no_influence` | state, then prove |
| `quantum.cyclic_support` | retire |
| `quantum.unitary_factorization` | keep open as the one research target |

Decisions confirmed by the maintainer on 2026-10-08.

## Context

- `lean/lean-toolchain` pins `leanprover/lean4:v4.32.0`; Mathlib rev
  `81a5d257c8e410db227a6665ed08f64fea08e997` (`lean/lake-lock.json`).
- The quantum Lean tree is `lean/DeepCausalityFormal/Quantum/`: `PartialTrace.lean`, `Choi.lean`,
  `PartialTraceCounterexample.lean`, `Abstraction.lean`, all on a pair-indexed matrix model
  (`Matrix (α × β) (α × β) R`, `R` a `CommRing`). `Choi.lean` already defines `applyChoi` and
  `choiOf`.
- Status documents: `deep_causality_quantum/LEAN_QUANTUM.md` and the quantum section of
  `lean/THEOREM_MAP.md`. The live spec is `openspec/specs/quantum-formalization/spec.md`
  (requirement "The in-scope quantum causal-model theorems").
- The website lists the targets in `website/quantum/src/data/formalization.ts` (`deferred`) and
  counts them in `website/quantum/src/data/evidence.ts` (`TESTS.deferred = 7`).

## What the pinned Mathlib already provides

| Need | Mathlib lemma | File |
|---|---|---|
| Matrix as a sum of matrix units | `Matrix.matrix_eq_sum_single` | `Mathlib/Data/Matrix/Basis.lean:175` |
| Mixed product of Kronecker products | `Matrix.mul_kronecker_mul` | `Mathlib/LinearAlgebra/Matrix/Kronecker.lean:382` |
| Diagonal matrices multiply diagonally | `Matrix.diagonal_mul_diagonal` | `Mathlib/Data/Matrix/Mul.lean:381` |
| Orthomodular law for subspaces | `Submodule.sup_orthogonal_inf_of_hasOrthogonalProjection` | `Mathlib/Analysis/InnerProductSpace/Projection/Submodule.lean:36` |
| Orthocomplement is an involution | `Submodule.orthogonal_orthogonal` | same file, line 56 |
| `K ⊓ Kᗮ = ⊥` | `Submodule.inf_orthogonal_eq_bot` | `Mathlib/Analysis/InnerProductSpace/Orthogonal.lean:98` |

Wedderburn–Artin is present (`Mathlib/RingTheory/SimpleModule/WedderburnArtin.lean`), and C*-algebra
basics are under `Mathlib/Analysis/CStarAlgebra/`. The commutant and direct-sum decomposition of a
finite-dimensional C*-algebra, which Lorenz and Barrett's Theorem 1 rests on, is not.

## Statements to settle in the design

- **CJ reconstruction.** For every `R`-linear `E : Matrix α α R →ₗ[R] Matrix β β R`,
  `applyChoi (choiOf E) = E` pointwise. Proof: expand the input by `matrix_eq_sum_single`, push `E`
  through the sum by linearity.
- **Orthomodular.** For a finite-dimensional inner product space over `ℂ`, the lattice of
  subspaces with `ᗮ` is bounded, `ᗮ` is an antitone involution with `K ⊓ Kᗮ = ⊥` and
  `K ⊔ Kᗮ = ⊤`, and `K₁ ≤ K₂ → K₁ ⊔ (K₁ᗮ ⊓ K₂) = K₂`. Plus a witness in `ℂ²` that distributivity
  fails, using the spans of `|0⟩`, `|1⟩` and `|+⟩`, the triple the Rust carrier documents.
- **Markov commutativity.** Lorenz 2022, Definition 3.3, is a definition. The theorem: factors on
  disjoint legs commute (`(A ⊗ 1)(1 ⊗ B) = (1 ⊗ B)(A ⊗ 1)` on the pair-indexed model), and the
  product of a pairwise-commuting finite family does not depend on its order.
- **Classical embedding.** Diagonal factors commute pairwise, so every family of diagonal factors
  meets the Markov condition, and the Kronecker product of diagonal matrices is diagonal.
- **No influence.** Lorenz and Barrett 2021, Definition 1, is a definition. The theorem: for a
  linear map on the bipartite model, the marginal on D factors through `Tr_A` exactly when it does
  not depend on the A input for product inputs of unit trace. The forward direction uses
  `partialTraceRight_kron`; the converse uses that product matrices span the space.

## Constraints

- Zero `sorry` in every new or touched file. Each proved id gets a `THEOREM_MAP.md` row and a Rust
  witness; a witness that does not exist yet is written in `deep_causality_quantum/tests/`.
- `cyclic_support` is retired because the crate refuses cyclic structures at `build()` by decision
  and no statement exists. The separability fact it would rest on is already proved as
  `core.context_graph.acyclicity_separable`.
- The website must state the remaining target and why it is hard, and drop the count of seven.

## Risks and gaps

- The Lean dependency fetch ships a trimmed Mathlib olean set (1,918 modules). The orthomodular
  proof imports inner-product-space modules that may sit outside it; if so, the archive is
  regenerated.
- `LEAN_QUANTUM.md` and `THEOREM_MAP.md` state Lean and Mathlib 4.15.0; the toolchain is v4.32.0.
- The live spec names `deep_causality_do_calculus`, which is not a workspace crate.
- The comment in `Choi.lean` names `stdBasisMatrix`; the definitions already use `Matrix.single`.
- The no-influence statement is the least settled; the design fixes it before any proof starts.
