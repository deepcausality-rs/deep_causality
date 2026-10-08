## 1. Toolchain check

- [x] 1.1 Confirm each Mathlib lemma the design names exists at rev `81a5d257` with `#check` in a scratch file: `matrix_eq_sum_single`, `mul_kronecker_mul`, `diagonal_mul_diagonal`, `List.Perm.prod_eq'`, `Submodule.orthogonal_orthogonal`, `Submodule.inf_orthogonal_eq_bot`, `Submodule.sup_orthogonal_of_hasOrthogonalProjection`, `Submodule.sup_orthogonal_inf_of_hasOrthogonalProjection`, `Submodule.orthogonal_le`
- [x] 1.2 Build a stub `Quantum/Orthomodular.lean` that only imports the inner-product-space modules; if the trimmed olean set lacks them, regenerate the lock and archive with `scripts/lean_lock.sh` and confirm `bazel test //lean/...` still passes

## 2. CJ reconstruction

- [x] 2.1 Prove `applyChoi_choiOf` in `Quantum/Choi.lean` (design D2) and replace the deferral comment
- [x] 2.2 Add `choi_tests.rs :: test_choi_reconstruction_recovers_the_channel` on a channel with a non-diagonal Choi operator
- [x] 2.3 Add the `quantum.choi.reconstruction` row to `lean/THEOREM_MAP.md`

## 3. Orthomodular lattice

- [x] 3.1 Prove the bounded-lattice, orthocomplement and orthomodular laws in `Quantum/Orthomodular.lean` (design D3)
- [x] 3.2 Prove the distributivity failure for `span{e₀}`, `span{e₁}`, `span{e₀ + e₁}` in `EuclideanSpace ℂ (Fin 2)`
- [x] 3.3 Add `tests/formalization_lean/orthomodular_tests.rs` with the orthomodular law and the documented triple on the `Projection` carrier, and register it in `mod.rs`
- [x] 3.4 Add the `quantum.verdict.orthomodular` row and update the `core.verdict.carriers` row, which calls the projection lattice "planned"

## 4. Markov commutativity and the classical embedding

- [x] 4.1 Prove the mixed-product law `kron_mul_kron` on the local `kron` (design D1)
- [x] 4.2 Prove `kron_one_commute_one_kron` and `pairwise_commute_prod_perm` in `Quantum/Markov.lean` (design D4)
- [x] 4.3 Prove `diagonal_commute`, the pairwise corollary and `kron_diagonal` in `Quantum/ClassicalEmbedding.lean` (design D5)
- [x] 4.4 Add `tests/formalization_lean/qcm_theorem_tests.rs` with the free-commutation, order-independence and diagonal-commutation witnesses, and register it in `mod.rs`
- [x] 4.5 Add the `quantum.markov_commutativity` and `quantum.classical_embedding` rows

## 5. No influence

- [x] 5.1 Define `partialTraceLeft` and prove `partialTraceLeft_kron` (design D6)
- [x] 5.2 Prove the product-basis expansion `matrix_eq_sum_kron_single`
- [x] 5.3 Define `NoInfluence` and prove `noInfluence_iff` in `Quantum/NoInfluence.lean`
- [x] 5.4 Add the no-influence witnesses to `qcm_theorem_tests.rs`: a product channel passes, a swap carrying A to D fails
- [x] 5.5 Add the `quantum.no_influence` row

## 6. Wiring and gates

- [x] 6.1 Import every new file from `lean/DeepCausalityFormal.lean` and add it to the Lean Bazel targets
- [x] 6.2 Run `bazel test //lean/...` and confirm zero `sorry` in `lean/DeepCausalityFormal/Quantum/`
- [x] 6.3 Run `bazel test //deep_causality_quantum/...` and `cargo test -p deep_causality_quantum --all-features`
- [x] 6.4 Run the theorem-map traceability check and confirm every new row resolves to its Lean declaration and Rust test

## 7. Status documents

- [x] 7.1 Rewrite the deferred section of `deep_causality_quantum/LEAN_QUANTUM.md`: the new proved rows, `unitary_factorization` as the one open target with its reason, no `cyclic_support`
- [x] 7.2 Rewrite the closing paragraph of the quantum section of `lean/THEOREM_MAP.md` the same way
- [x] 7.3 Correct the Lean and Mathlib version in both files to v4.32.0
- [x] 7.4 Remove the `deep_causality_do_calculus` reference when the change archives into the live spec
- [x] 7.5 Update `website/docs/src/content/docs/formalization/quantum.md`: a row per proved id and the one-target pointer

## 8. Quantum website

- [x] 8.1 Add the new theorems to `src/data/formalization.ts` and reduce `deferred` to `unitary_factorization` with its reason
- [x] 8.2 Set `TESTS.deferred` to 1 and `TESTS.proved` to the new total in `src/data/evidence.ts`
- [x] 8.3 Rewrite the proof page's deferred section around the one open target and update its theorem-count prose
- [x] 8.4 Update the home page's evidence row and the `check_markov` evidence line in `src/data/checks.ts`
- [x] 8.5 Build the site, run `scripts/sitecheck.py --strict` and `pnpm check`
