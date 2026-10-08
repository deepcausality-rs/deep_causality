## Context

The quantum Lean tree, `lean/DeepCausalityFormal/Quantum/`, proves fourteen statements on a
pair-indexed matrix model: `Matrix (α × β) (α × β) R` over a commutative ring `R`, with `kron` and
`partialTraceRight` defined in `PartialTrace.lean` and `applyChoi` and `choiOf` in `Choi.lean`.
Seven targets are stated and unproved. `LEAN_QUANTUM.md` attributes all seven to missing Mathlib
machinery; a search of the pinned Mathlib (rev `81a5d257`, Lean v4.32.0) shows the lemmas for five
of them already exist. The source note is
`openspec/notes/quantum-lean-deferred/deferred-proofs.md`; the maintainer confirmed the decisions on
2026-10-08.

## Goals / Non-Goals

**Goals:**

- Prove the CJ reconstruction, the orthomodular lattice, Markov commutativity, the classical
  embedding and no-influence, each with zero `sorry`, a `THEOREM_MAP.md` row and a Rust witness.
- Retire `quantum.cyclic_support` and record `quantum.unitary_factorization` as the one open
  target, with its reason.
- Bring the status documents, the docs site and the quantum website in line with the result.

**Non-Goals:**

- Proving `quantum.unitary_factorization`.
- Formalizing the bijection between Rust projections (Hermitian idempotent matrices) and
  subspaces. The Lean statement covers the subspace lattice the Rust carrier's ranges form.
- Changing any Rust library API.

## Decisions

### D1. Stay on the pair-indexed matrix model

New theorems use `Matrix (α × β) (α × β) R` and the existing `kron` and `partialTraceRight`, so they
compose with the fourteen proved statements. The mixed-product law `kron_mul_kron`,
`(A ⊗ B)(C ⊗ D) = (AC) ⊗ (BD)`, is proved directly on `kron`. Mathlib's `Matrix.mul_kronecker_mul`
would need `Mathlib.LinearAlgebra.Matrix.Kronecker`, which `PartialTrace.lean`'s import policy keeps
out, and the direct proof is four lines.

*Alternative:* Mathlib's `TensorProduct` of modules. Rejected: it shares nothing with the existing
files and makes every statement a change of representation.

### D2. The CJ reconstruction by basis expansion

`applyChoi (choiOf E) A = E A` follows by writing `A` as `∑ i j, A i j • single i j 1`
(`Matrix.matrix_eq_sum_single`), pushing `E` through the sum by `map_sum` and `map_smul`, and
matching entries with the definition of `applyChoi`. The theorem is
`applyChoi_choiOf` in `Choi.lean`, id `quantum.choi.reconstruction`.

### D3. Orthomodular on Mathlib's subspace lattice

The statement lives in a new `Quantum/Orthomodular.lean` over a finite-dimensional
`InnerProductSpace ℂ E`, where every subspace has an orthogonal projection. The laws come from
`Submodule.orthogonal_orthogonal`, `Submodule.inf_orthogonal_eq_bot`,
`Submodule.sup_orthogonal_of_hasOrthogonalProjection` and
`Submodule.sup_orthogonal_inf_of_hasOrthogonalProjection`; antitonicity from
`Submodule.orthogonal_le`. The distributivity witness is stated for any two linearly independent
vectors `u` and `v`, with the lines through `u`, `v` and `u + v`: `u` lies in `K_u ⊓ (K_v ⊔ K_{u+v})`,
while `K_u ⊓ K_v = K_u ⊓ K_{u+v} = ⊥`. `|0⟩` and `|1⟩` in `ℂ²` are one instance. Stating it in
`EuclideanSpace ℂ (Fin 2)` would import `PiL2`, which grows the Mathlib closure from 2,319 to 8,639
modules.

*Alternative:* define projections as Hermitian idempotent matrices and prove the lattice laws on
them. Rejected: it rebuilds what Mathlib has, and the range map to subspaces is the standard
identification the Rust carrier already uses.

### D4. Markov commutativity as two lemmas

Lorenz 2022, Definition 3.3, defines the Markov condition; the theorem states what makes the check
sound on this model. `kron_one_commute_one_kron`: `kron A 1 * kron 1 B = kron 1 B * kron A 1`, from
`mul_kronecker_mul` and `one_mul`/`mul_one`. `pairwise_commute_prod_perm`: for lists `l` and `l'`
that are permutations of each other with `l.Pairwise Commute`, `l.prod = l'.prod`, from Mathlib's
`List.Perm.prod_eq'`. A new file `Quantum/Markov.lean` holds both.

### D5. The classical embedding through diagonal matrices

In `Quantum/ClassicalEmbedding.lean`: `diagonal_commute`, `Commute (diagonal d₁) (diagonal d₂)`, from
`Matrix.diagonal_mul_diagonal` and `mul_comm`; a corollary that any list of diagonal factors is
pairwise commuting, so D4's order independence applies; and `kron_diagonal`, the Kronecker product of
two diagonals is the diagonal of the product function, so tensoring classical tables stays classical.

### D6. No influence, stated algebraically

In `Quantum/NoInfluence.lean`. Define `partialTraceLeft` (trace out the first factor) with
`partialTraceLeft_kron : Tr_A (X ⊗ Y) = trace X • Y`. For a linear map
`U : Matrix (α × β) (α × β) R →ₗ[R] Matrix (γ × δ) (γ × δ) R`, define
`NoInfluence U := ∃ M : Matrix β β R →ₗ[R] Matrix δ δ R, ∀ ρ, Tr_C (U ρ) = M (Tr_A ρ)`.
The theorem `noInfluence_iff`, under `[Nonempty α]`:
`NoInfluence U ↔ ∀ X X' Y, trace X = trace X' → Tr_C (U (X ⊗ Y)) = Tr_C (U (X' ⊗ Y))`.
Forward: `partialTraceLeft_kron` makes the two `Tr_A` values equal. Converse: fix `e = single i₀ i₀ 1`
(trace 1), set `M Y := Tr_C (U (e ⊗ Y))`, show agreement on each `X ⊗ Y` by comparing it with
`(trace X • e) ⊗ Y`, then extend to every `ρ` because the matrix units `single (i,k) (j,l) 1` are
Kronecker products `single i j 1 ⊗ single k l 1`.

*Alternative:* Lorenz and Barrett's unitary formulation with states and channels. Rejected: it needs
unitarity and positivity on top of the same algebra, and the linear statement covers the unitary
case as an instance.

### D7. Witnesses under `tests/formalization_lean/`

The docs spec requires witnesses in `deep_causality_quantum/tests/formalization_lean/`. New tests:
`choi_tests.rs :: test_choi_reconstruction_recovers_the_channel`; a new `qcm_theorem_tests.rs` with
the free-commutation, order-independence, diagonal-commutation and no-influence witnesses (a product
channel passes, a swap that carries A to D fails); a new `orthomodular_tests.rs` that reruns the
orthomodular law and the documented distributivity failure on the `Projection` carrier. Each file is
registered in `mod.rs` and covered by the crate's Bazel test target.

### D8. Retirement and the open target

`cyclic_support` is removed from the live spec, `LEAN_QUANTUM.md`, `THEOREM_MAP.md`, the website's
`deferred` list and the docs page. No Lean file exists for it. `unitary_factorization` stays in each
of those places as the one open target, with the reason: Lorenz and Barrett's Theorem 1 rests on the
commutant and direct-sum decomposition of finite-dimensional C*-algebras; Mathlib has Wedderburn–Artin
and C*-algebra basics but not that decomposition.

## Risks / Trade-offs

- [The trimmed Mathlib olean set may omit `Analysis.InnerProductSpace.*`] → Build `Orthomodular.lean`
  first; if an import is missing, regenerate the lock and archive with `scripts/lean_lock.sh` before
  the other proofs.
- [A lemma name differs at this Mathlib rev] → Each decision names its lemma; confirm with
  `#check` before writing the proof, and adapt the name, not the statement.
- [The converse of `noInfluence_iff` is the longest proof] → Prove the product-basis expansion as a
  standalone lemma (`matrix_eq_sum_kron_single`) so the main proof stays short.
- [D3 states the law on subspaces, the Rust carrier works on matrices] → The `THEOREM_MAP.md` row and
  the Rust witness name the range correspondence; the witness checks the laws on the carrier itself.

## Migration Plan

No runtime change. The spec retirement of `cyclic_support` is recorded in the commit message. Website
and docs counts change from seven open targets to one.

## Open Questions

- None blocking. The id `quantum.choi.reconstruction` is new; if the map prefers the existing
  `quantum.choi.*` naming with a different suffix, rename before the row lands.
