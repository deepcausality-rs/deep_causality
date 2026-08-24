The crate is built test-first. Phase 0 prepares the tower, phases 1–4 produce a fully tested
`deep_causality_linear` that nothing depends on yet, and only then does phase 5 repoint a consumer.
Do not start a phase until the previous one's exit condition holds.

## 0. Prepare the tower

The linear crate cannot declare its own scalar, so 𝔽₂ has to exist before phase 1 can declare the
packed representation. Exit condition: the tower carries `Gf2` and the characteristic refinements,
the workspace is green, and no `Field` bound that halves can reach 𝔽₂.

- [ ] 0.1 Add `Gf2` to `deep_causality_num`: `Add`, `Sub`, `Neg`, `Mul`, `Div`, `DivAssign`, `Zero`, `One`, `PartialEq`, `Eq`, `Clone`, `Copy`, `Debug`, `Display`, all arithmetic mod 2
- [ ] 0.2 Implement its law markers in `deep_causality_algebra`: `Associative<Additive>`, `Associative<Multiplicative>`, `Commutative<Additive>`, `Commutative<Multiplicative>`, `Distributive`, `Annihilating`
- [ ] 0.3 Implement `IntegralDomain for Gf2` deliberately, per the tower's per-type rule; confirm `Field` arrives through the blanket and is not written by hand
- [ ] 0.4 Confirm by compile probe that `Gf2` does NOT reach `RealField`, `Normed`, `NormedScalar`, `ConjugateScalar` or `EuclideanDomain`
- [ ] 0.5 Add `Characteristic` with the characteristic as an associated constant — 0 for ℚ, ℝ, ℂ; `p` for a finite field
- [ ] 0.6 Add `CharacteristicZero: Field` with per-type impls for `f32`, `f64`, `Float106`, `Complex<T>` and `Rational<T>`; document that it is an unverifiable promise like every other law here
- [ ] 0.7 Add `FiniteField: Field` carrying `ORDER`; implement for `Gf2` with order 2 and characteristic 2; document that `q = p^k` and that 𝔽₄ has order 4 and characteristic 2, so the two are different questions
- [ ] 0.8 Document on both traits that they are disjoint by definition but do not partition the fields, naming 𝔽ₚ(x) as the case that is in neither
- [ ] 0.9 Rebound the four `Field`-bounded sites that divide by `one + one` on `CharacteristicZero`: `multivector/src/types/multifield/algebra/mod.rs` `commutator_geometric`, `multivector/src/types/multivector/ops/ops_product_impl.rs:316`, `multivector/src/types/multifield/ops/conversions.rs:139`, and the `svd_decomp` site at `tensor/.../tensor_svd_decomp/mod.rs:64`
- [ ] 0.10 Let the compiler enumerate any site 0.9 missed; rebound each and record the count against the sixteen `one() + one()` sites the survey found
- [ ] 0.11 Add a compile-fail test that halving over `Gf2` is rejected and that the error names `CharacteristicZero`
- [ ] 0.12 Add `Distributive` and `Annihilating` for `CsrMatrix<T>`, closing the two markers that keep it at `AbelianGroup`; confirm `CsrMatrix<f64>: Ring` then holds
- [ ] 0.13 Add `impl Module<S> for CsrMatrix<T>`, now reachable, and confirm the tower sees the scalar multiplication `arithmetic/mod.rs:283` already implements
- [ ] 0.14 Bump `deep_causality_num` and `deep_causality_algebra`, repin dependents, and confirm `bazel test //...` is green before phase 1 starts

## 1. Declare the API, implement nothing

Exit condition: the crate compiles, every public item is reachable, every call panics as
unimplemented.

- [ ] 1.1 Create `deep_causality_linear` with `[lints] workspace = true`, `no_std` + `alloc` feature parity with `deep_causality_sparse`, dependencies on `deep_causality_num`, `deep_causality_algebra`, `deep_causality_haft` only, and dev-dependencies on `deep_causality_num_complex` and `deep_causality_num_rational` — the suite instantiates `Complex<f64>` for the Hermitian inner product and `Rational<i64>` for the unordered-field and rank-agreement scenarios
- [ ] 1.2 Declare the access traits: read (`rows`, `cols`, `get` by value), row operations (`swap_rows`, `scale_row`, `axpy_rows`, overridable `pivot_in_column`), build (`zeros`, `set`, default `identity`)
- [ ] 1.3 Declare the four containers — CSR, dense row-major, dense vector, bit-packed 𝔽₂ over a `NaturalNumber` word — with their constructors and accessors, bodies `todo!()`
- [ ] 1.4 Declare the algorithm surface: `rref`, `rank`, `kernel_basis`, `image_basis`, `determinant`, `solve`, the LU factorisation as a reusable value, forward/backward substitution, the CG solvers, and the six decompositions
- [ ] 1.5 Declare the vector surface: element access, scale, add, sub, dot, Hermitian inner product, outer product, slice round-trip, and matrix–vector for all three matrix representations
- [ ] 1.6 Declare the norms: vector 1/2/∞, matrix 1/∞/Frobenius, each in exactly one place, bounded on `NormedScalar`
- [ ] 1.7 Declare the integer surface: fraction-free determinant and exact rank over `EuclideanDomain`, plus the ring operations over `CommutativeRing`
- [ ] 1.8 Band every declaration on the weakest tower trait that makes it correct — `CommutativeSemiring` / `CommutativeRing` / `IntegralDomain` / `EuclideanDomain` / `Field` / `CharacteristicZero` / `NormedScalar` / `RealField` — and document on each what property the bound supplies
- [ ] 1.8a Bound nothing on an ad-hoc `core::ops` bundle. The code being moved does this throughout — `mat_mult_impl` takes `T: Copy + Clone + Mul<Output = T> + Zero + PartialEq + Default`, `transpose_impl` takes `T: Copy + Zero`, `vec_mult_impl` takes `T: Copy + Zero + Add + Mul` — and each is a semiring spelled longhand. Restate them as tower traits, keeping `Copy`/`Clone`/`Default` where the representation needs them
- [ ] 1.8b Record every bound loosened from `Field` or `RealField` with the number set it newly admits, so phase 2 can write the test that instantiates it
- [ ] 1.9 Confirm the crate declares no scalar trait, marker or newtype of its own
- [ ] 1.10 Declare an HKT witness per container, matching the trait set `CsrMatrixWitness` implements (`HKT`, `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad`, `Adjunction`)
- [ ] 1.10a Declare the tower impls per container: each matrix reaches `Ring` via `AbelianGroup` + `MulMonoid` + `Distributive` + `Annihilating`, with `Associative<Multiplicative>` and deliberately without `Commutative<Multiplicative>`; the vector reaches `AbelianGroup` with the additive markers only
- [ ] 1.10b Declare `Module<R>` for every container, the vector included — the tower's name for a vector space, and the bound that admits ℤ where `Field` would not
- [ ] 1.10c Document at each impl site any tower trait a container stops short of, and why
- [ ] 1.11 Declare the error types and the conversions among representations
- [ ] 1.12 Add `BUILD.bazel`; confirm `cargo build` and `bazel build //deep_causality_linear/...` both succeed
- [ ] 1.13 Confirm no algorithm exists yet: every public function body is `todo!()` or a trivial accessor
- [ ] 1.14 Confirm the band assignment compiles as intended: `rref` over `i64` must FAIL to compile; `determinant` over `i64` must compile; matrix subtraction over `u64` must FAIL to compile; `rref` over `Gf2` must compile; any operation that halves must FAIL to compile over `Gf2`

## 2. Write the full suite against the unimplemented API

Exit condition: every test compiles, every test fails, and every failure is the unimplemented panic.

- [ ] 2.1 Build the test tree mirroring `src` file for file, every module registered upward, every directory declared in `tests/BUILD.bazel`
- [ ] 2.2 Put every shared test helper under `src/utils_tests/` — never inside `tests/`, which Bazel cannot reach — and give each helper its own test at `tests/utils_tests/<name>_tests.rs`, since helpers in `src` are library code and count toward coverage
- [ ] 2.3 Write one test per scenario in `linear-matrix-representations`, `linear-dense-algorithms`, `linear-f2-algebra` and `linear-crate-identity`; record the scenario → test mapping in a committed `SCENARIOS.md`
- [ ] 2.4 Cover the enumerated corner cases: 0×0, 0×n, n×0, 1×1, non-square both ways, zero row, zero column, singular, non-singular with a zero `(0,0)`, rank-deficient, column count not a multiple of the word width, same matrix at two word widths, `{-1,0,1}` reduced mod 2, an entry outside `{0,1}` offered to the packed constructor, an empty CSR row, an out-of-shape index, a near-zero float pivot with a larger one below
- [ ] 2.5 Write the Cayley-Menger regression: the 5×5 CM matrix of a regular unit tetrahedron must give `det = 4.0` and `vol = √2⁄12` (`prototype/cm_det.py` reproduces both, and shows an unpivoted elimination returning zero)
- [ ] 2.6 Write the 𝔽₂ oracle tests: kernel vectors annihilate and number `cols − rank`; the image basis has `rank` elements and spans the columns; two word widths agree on rank and pivot columns
- [ ] 2.7 Write the scalar-band tests: the three compile-fail cases from 1.14 as `trybuild`-style or documented negative tests, and one positive test per band showing the admitted scalars
- [ ] 2.8 Write the integer tests: fraction-free determinant equals the exact rational answer with no float appearing; exact rank on a matrix whose singular values straddle `1e-5`; the three ranks disagreeing on one matrix
- [ ] 2.9 Write the vector tests: dot against a manual sum, outer product shape, length mismatch rejected, slice round-trip, Hermitian inner product real and non-negative on a complex vector, norms of `[3, -4]` giving 7 / 5 / 4, zero vector giving no `NaN`
- [ ] 2.10 Write the solve tests: known system, singular rejected, zero `(0,0)` handled, one factorisation applied to three right-hand sides, triangular substitution against the general path, zero diagonal rejected, wrong triangle rejected
- [ ] 2.11 Write the HKT law tests per witness: functor identity and composition, monad left/right identity and associativity, comonad `extend(extract)`, and shape preserved by `fmap`
- [ ] 2.11a Write the tower-membership tests: each container admitted by a function bounded on `Ring` and by one bounded on `Module<R>`; each matrix REJECTED by one bounded on `CommutativeRing`, because matrix multiplication does not commute
- [ ] 2.11b Write one instantiation test per bound recorded in 1.8b, calling the loosened operation at the number set it newly admits — `i64` for every operation moved off `Field`, `u64` for every operation moved to `CommutativeSemiring`
- [ ] 2.12 Write the delegation tests: each decomposition's result and error variant, to be compared against `CausalTensor`'s current output in phase 5
- [ ] 2.13 Port the sparse crate's 1,916 lines of tests, adjusted to the new paths
- [ ] 2.14 Run the suite: confirm every test fails, and that each failure is the unimplemented panic rather than a compile error, a missing import, or an unrelated panic
- [ ] 2.15 Confirm the suite needed no addition to the public surface to compile — if it did, that is an API design finding: record it and revise phase 1

## 3. Verify the suite before trusting it

Exit condition: the suite is shown to reject every known defect class, and the tree is clean again.

- [ ] 3.1 Stub a deliberately unpivoted elimination; confirm a Cayley-Menger test fails
- [ ] 3.2 Stub a packed row update that skips one word, then one that repeats a word; confirm a test fails in each case
- [ ] 3.3 Replace an 𝔽₂ exactness check with a tolerance comparison; confirm a test fails
- [ ] 3.4 Perturb a reported rank by ±1; confirm a test fails in both directions
- [ ] 3.5 Widen a band deliberately — bound `rref` on `Field` where the body needs a pivot rule, or drop `EuclideanDomain` to `CommutativeRing` on the integer determinant; confirm a test or a compile-fail case catches it
- [ ] 3.6 Break one HKT law in one witness; confirm a law test fails
- [ ] 3.6a Claim `Commutative<Multiplicative>` on a matrix container; confirm a test fails, because the claim is false for matrix multiplication
- [ ] 3.6b Remove one container's `Distributive` impl; confirm the `Ring` admission test fails — this is the defect the crate inherits, so the suite must catch it
- [ ] 3.7 Replace `solve` with invert-then-multiply; confirm the ill-conditioned residual test fails
- [ ] 3.8 Check the scenario → test mapping is complete; every scenario in the four capabilities names at least one test
- [ ] 3.9 Check the corner-case list against the suite; every case names a test
- [ ] 3.10 Revert every deliberate defect; confirm the tree is clean

## 4. Implement until the suite is green

Exit condition: suite green under both build systems, full coverage, clippy clean.

- [ ] 4.1 Implement the dense representation and its trait impls, with magnitude pivoting for float scalars
- [ ] 4.2 Implement the bit-packed 𝔽₂ representation with whole-word row updates; it overrides no pivot rule
- [ ] 4.3 Implement CSR: the read trait only. Do not implement row operations for it — an axpy changes the non-zero pattern — and document that in the module header
- [ ] 4.4 Implement the conversions, fallible where the target cannot hold the source's values
- [ ] 4.5 Implement `rref`, `rank`, `kernel_basis`, `image_basis`, `solve`, generic over the row-operation trait, naming no concrete representation or scalar
- [ ] 4.6 Implement `determinant` over `Field`: pivot by column search always; closed forms at order ≤ 3, elimination at order ≥ 4
- [ ] 4.7 Implement the integer path over `EuclideanDomain`: Bareiss fraction-free determinant and exact rank, using `div_euclid` and `normalize`, with no value converted to a float
- [ ] 4.8 Implement the vector, its products and the norms; implement matrix–vector for all three representations, sparse without densifying
- [ ] 4.9 Implement `solve`, the reusable LU factorisation carrying its permutation, and forward/backward substitution; document on `inverse` that `solve` is preferred for `A⁻¹b`
- [ ] 4.10 Implement the HKT witnesses; verify the laws hold at representative values
- [ ] 4.10a Implement the tower impls per 1.10a–1.10c; confirm every container is admitted by a `Ring` bound and a `Module<R>` bound, and that no matrix claims `Commutative<Multiplicative>`
- [ ] 4.10b Implement the packed 𝔽₂ representation against `Gf2` from `deep_causality_num`; confirm this crate declares no scalar type of its own
- [ ] 4.11 Move `CsrMatrix`, its arithmetic and ops, `solver/cg.rs`, `extensions/ext_hkt.rs` and the errors from `deep_causality_sparse`
- [ ] 4.12 Move the bodies of `svd` (117), `svd_decomp` (170), `svd_truncated` (375), `qr` (145), `eigen` (158) and `inverse` (123) from `deep_causality_tensor`
- [ ] 4.13 Fix implementation, not tests, wherever the suite disagrees; if a test's assertion is wrong because the API is wrong, change the API and say so
- [ ] 4.14 `cargo llvm-cov`: full line coverage on every added file, unreachable lines excepted and justified
- [ ] 4.15 Clippy clean with no new `#[allow]`; `cargo test` and `bazel test //deep_causality_linear/...` both green

## 5. Migrate downstream, gated on phase 4

Exit condition: the workspace builds against the new crate and the old name still works.

- [ ] 5.1 Confirm phase 4's exit condition holds before touching any consumer
- [ ] 5.2 Add `deep_causality_linear` as a dependency of `deep_causality_tensor`; implement the read trait for `CausalTensor` there — this impl cannot live anywhere else (E0117)
- [ ] 5.3 Reduce `CausalTensor`'s inherent methods and the `Tensor` trait members at `traits/tensor.rs:435,439` to delegations; keep every signature, return shape and error variant
- [ ] 5.4 Record the tensor benchmark baseline, re-run after delegation, diff; record both figures with the machine
- [ ] 5.5 Move `ext_iso.rs` and `CsrFromTensorError` into `deep_causality_tensor` unconditionally; delete the `tensor-iso` feature, its `#[cfg]` gates, the `"tensor-iso"` entries in `deep_causality_sparse/BUILD.bazel:10` and `tests/BUILD.bazel:58`, and `features = ["tensor-iso"]` on `examples/mathematics_examples/Cargo.toml:23`
- [ ] 5.6 Reduce `deep_causality_sparse/src/lib.rs` to re-exports of `deep_causality_linear`; confirm the public surface matches its last independent release item for item
- [ ] 5.7 Write the retirement notice at the top of `deep_causality_sparse/README.md`, naming the successor and stating that the crate receives no further development
- [ ] 5.8 Switch the 102 import sites: topology 61 (32 `src`, 29 `tests`), examples 10, physics 2 (`kernels/mhd/ideal.rs:11`, `kernels/mhd/grmhd.rs:11`), and the 29 inside the crate being moved
- [ ] 5.9 Retarget the 35 Bazel label references across the 8 `BUILD.bazel` files that name the old crate
- [ ] 5.10 Resolve the `deep_causality_cfd` discrepancy: `BUILD.bazel:30` declares a dependency `Cargo.toml` does not — decide which is correct and make both agree
- [ ] 5.11 Register the crate in `build/scripts/sbom.sh` and commit its generated `*_sbom.spdx.json` + `.sha`
- [ ] 5.12 Update `AGENTS.md` §Project Structure and §Project Dependencies, `README.md:268`, `website/web/src/pages/overview/index.astro` (2 sites), `website/docs/…/getting-started/install.md`, `website/docs/…/concepts/uniform-math.md`
- [ ] 5.13 Leave the 34 files under `openspec/changes/archive/` unchanged; confirm by diff
- [ ] 5.14 Rebuild the 8 in-workspace and 7 example tensor dependents with no source edit; confirm results unchanged
- [ ] 5.15 `cargo test --workspace` and `bazel test //...` green

## 6. Retire the duplication in topology and multivector

Exit condition: one determinant, one rank helper, one Euclidean norm, exact 𝔽₂ homology, G-01 and
G-02 closed.

- [ ] 6.1 Replace `regge_geometry/curvature.rs:275` `det_recursive` and its `submatrix` helper with the shared determinant
- [ ] 6.2 Replace `manifold/geometry/mod.rs:145` `determinant_impl` with the shared determinant
- [ ] 6.3 Replace `simplicial_complex/lazy_hodge_star.rs:97` `gaussian_determinant` with the shared determinant
- [ ] 6.4 Confirm the Cayley-Menger volumes are unchanged: `regge_geometry` (5×5, tetrahedron) and `manifold/geometry` (`k + 2`) both feed matrices with a zero `(0,0)` entry
- [ ] 6.5 Diff the topology suite before and after 6.1–6.4; investigate every changed value rather than re-baselining
- [ ] 6.6 Replace `chain_complex_impl.rs:94` `rank_of_csr` and `cell_complex/mod.rs:172` `rank_of_matrix` with one implementation
- [ ] 6.7 Route `betti_number` through an exact rank — 𝔽₂ for complexes read as codes, exact integer otherwise — and make the choice of field explicit at the call site rather than a global default
- [ ] 6.8 Confirm every complex currently under test reports the Betti numbers it reported before
- [ ] 6.9 Confirm no floating-point tolerance remains on any rank path used for homology
- [ ] 6.10 Mark G-01 and G-02 closed in `openspec/notes/quantum/qcl-gaps.md`; record the implementing crate in the owner field
- [ ] 6.11 Add a Lean theorem and Rust witness for mod-2 rank–nullity if the formalization layer covers it; otherwise record the omission in the crate's `LEAN_*.md` and add the crate to the `formalization.yml` allowlist only when a witness exists
- [ ] 6.12 Route `MultiVectorL2Norm::norm_l2` and `normalize_l2` (`multivector/src/traits/l2_norm.rs:21,30`, impl at `multivector/src/types/multivector/api/mod.rs:117`) through the vector 2-norm; keep the trait as the multivector-facing name if it earns one, the way `ScalarEval` does over `Normed`
- [ ] 6.13 Route `CausalMultiField::squared_magnitude` (`multivector/src/types/multifield/algebra/mod.rs`) through the vector squared 2-norm; it hand-writes `Σ *val * *val`, which is the squared modulus only because the impl is bounded on `RealField`, and a later widening would make it silently wrong for a complex scalar
- [ ] 6.14 Decide `BatchedMatMul` (`multivector/src/types/multifield/ops/batched_matmul.rs`, 62 lines) explicitly: it batches rank-3 slices, which is the tensor surface rather than the matrix surface. Record the decision either way; do not leave it unexamined
- [ ] 6.15 Leave `ScalarEval` alone and record why: `multivector/src/extensions/scalar_eval/mod.rs` is already a single blanket over `deep_causality_algebra::Normed`, added only for the `Sum` bound. It is a facade over the tower, not a second definition
- [ ] 6.16 Diff the `deep_causality_multivector` suite before and after 6.12–6.14; investigate every changed value rather than re-baselining
- [ ] 6.17 Correct the construction census in `openspec/notes/linear/deep-causality-linear.md`: the `deep_causality_multivector` row reads "0 direct", and its `src` has 13 `CausalTensor::from_slice`/`from_shape_fn` sites, at least two of them rank-2 (`alias/alias_hilbert_state.rs`). Re-count that row by the same method used for the other six

## 7. Publish

- [ ] 7.1 Publish `deep_causality_linear` 0.1.0 first — release-plz strips path dependencies when verifying publish tarballs, so each dependent resolves the published API of the crate below it
- [ ] 7.2 Publish the final `deep_causality_sparse` carrying the re-exports and the retirement notice
- [ ] 7.3 Publish `deep_causality_tensor`, `deep_causality_topology`, `deep_causality_physics` in dependency order
- [ ] 7.4 Verify a previously published dependent still resolves and compiles from crates.io
- [ ] 7.5 Confirm nothing was yanked at any point

## 8. Out of scope, recorded so it is not lost

- [ ] 8.1 File separately: `deep_causality_physics` `gr_utils.rs:114` `invert_3x3` and `adm_state.rs:126` `inverse_spatial_metric` are the same function with singularity thresholds 100× apart (`1e-14` against `1e-12`), the latter compared in `f64` and so lossy for `Float106`. Merging them is a `pub(crate) fn` inside physics and needs no crate boundary. Do not do it here
- [ ] 8.2 File separately: `CausalMultiField::inverse` (`multivector/src/types/multifield/algebra/mod.rs`) documents itself as "Uses matrix inverse for each cell"; the body calls the multivector reversion inverse. A doc/code mismatch inside multivector, unrelated to this change
- [ ] 8.3 File separately: `CausalTensor::matmul` is bounded `T: Ring + Copy + Default + PartialOrd` (`tensor_product/mod.rs:13`). Matrix multiplication needs no ordering, so `PartialOrd` is an over-bound on a published surface. Record the number set it excludes before loosening it
