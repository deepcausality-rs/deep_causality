The crate is built test-first. Phase 0 prepares the tower, phases 1–4 produce a fully tested
`deep_causality_linear` that nothing depends on yet, and only then does phase 5 repoint a consumer.
Do not start a phase until the previous one's exit condition holds.

## 0. Prepare the tower

The linear crate cannot declare its own scalar, so 𝔽₂ has to exist before phase 1 can declare the
packed representation. Exit condition: the tower carries `Gf2` and the characteristic refinements,
the workspace is green, and no `Field` bound that halves can reach 𝔽₂.

- [x] 0.1 Add `Gf2` to `deep_causality_num`: `Add`, `Sub`, `Neg`, `Mul`, `Div`, `DivAssign`, `Zero`, `One`, `PartialEq`, `Eq`, `Clone`, `Copy`, `Debug`, `Display`, all arithmetic mod 2
- [x] 0.2 Implement its law markers in `deep_causality_algebra`: `Associative<Additive>`, `Associative<Multiplicative>`, `Commutative<Additive>`, `Commutative<Multiplicative>`, `Distributive`, `Annihilating`
- [x] 0.3 Implement `IntegralDomain for Gf2` deliberately, per the tower's per-type rule; confirm `Field` arrives through the blanket and is not written by hand
- [x] 0.4 Confirm by compile probe that `Gf2` does NOT reach `RealField`, `Normed`, `NormedScalar`, `ConjugateScalar` or `EuclideanDomain`
- [x] 0.5 Add `Characteristic` with the characteristic as an associated constant — 0 for ℚ, ℝ, ℂ; `p` for a finite field
- [x] 0.6 Add `DivisibleByIntegers: Field` with per-type impls for `f32`, `f64`, `Float106`, `Complex<T>` and `Rational<T>`; document that it is an unverifiable promise like every other law here
- [x] 0.7 Add `FiniteField: Field` carrying `ORDER`; implement for `Gf2` with order 2 and characteristic 2; document that `q = p^k` and that 𝔽₄ has order 4 and characteristic 2, so the two are different questions
- [x] 0.8 Document on both traits that they are disjoint by definition but do not partition the fields, naming 𝔽ₚ(x) as the case that is in neither
- [x] 0.9 Rebound the three `Field`-bounded sites that divide by `one + one` on `DivisibleByIntegers`, all in `deep_causality_multivector`: `types/multifield/algebra/mod.rs:163` `commutator_geometric`, `types/multifield/ops/conversions.rs:139`, and `types/multivector/ops/ops_product_impl.rs:316`
- [x] 0.10 Let the compiler enumerate any site 0.9 missed; rebound each and record the count against the sixteen `one() + one()` sites the survey found. The other thirteen are already excluded by a bound 𝔽₂ cannot reach — `RealField` (nine), `ConjugateScalar` (two) and `Real` (three, in `num_dual`) — each confirmed by compile probe rather than by reading the bound
- [x] 0.11 Add a compile-fail test that halving over `Gf2` is rejected and that the error names `DivisibleByIntegers`
- [x] 0.12 Add `Distributive` and `Annihilating` for `CsrMatrix<T>`, closing the two markers that keep it at `AbelianGroup`; confirm `CsrMatrix<f64>: Ring` then holds
- [x] 0.13 Confirm `CsrMatrix<T>: Module<S>` — it holds already, through the blanket at `algebra/module.rs:65`, which needs only `AbelianGroup` and the scalar multiplication at `arithmetic/mod.rs:283,321`. Writing an impl by hand is E0119. Record that `Ring` was the only rung missing
- [x] 0.14 Bump `deep_causality_num` and `deep_causality_algebra`, repin dependents, and confirm `bazel test //...` is green before phase 1 starts

## 1. Declare the API, implement nothing

Exit condition: the crate compiles, every public item is reachable, every call panics as
unimplemented.

- [x] 1.1 Create `deep_causality_linear` with `[lints] workspace = true`, `no_std` + `alloc` feature parity with `deep_causality_sparse`, dependencies on `deep_causality_num`, `deep_causality_algebra`, `deep_causality_haft` only, and dev-dependencies on `deep_causality_num_complex` and `deep_causality_num_rational` — the suite instantiates `Complex<f64>` for the Hermitian inner product and `Rational<i64>` for the unordered-field and rank-agreement scenarios
- [x] 1.2 Declare the access traits: read (`rows`, `cols`, `get` by value), row operations (`swap_rows`, `scale_row`, `axpy_rows`, overridable `pivot_in_column`), build (`zeros`, `set`, default `identity`)
- [x] 1.3 Declare the four containers — CSR, dense row-major, dense vector, bit-packed 𝔽₂ over a `NaturalNumber` word — with their constructors and accessors, bodies `todo!()`
- [x] 1.4 Declare the algorithm surface: `rref`, `rank`, `kernel_basis`, `image_basis`, `determinant`, `solve`, the LU factorisation as a reusable value, forward/backward substitution, the CG solvers, and the six decompositions
- [x] 1.5 Declare the vector surface: element access, scale, add, sub, dot, Hermitian inner product, outer product, slice round-trip, and matrix–vector for all three matrix representations
- [x] 1.6 Declare the norms: vector 1/2/∞, matrix 1/∞/Frobenius, each in exactly one place, bounded on `NormedScalar`
- [x] 1.7 Declare the integer surface: fraction-free determinant and exact rank over `EuclideanDomain`, plus the ring operations over `CommutativeRing`
- [x] 1.8 Band every declaration on the weakest tower trait that makes it correct — `CommutativeSemiring` / `CommutativeRing` / `IntegralDomain` / `EuclideanDomain` / `Field` / `DivisibleByIntegers` / `NormedScalar` / `RealField` — and document on each what property the bound supplies
- [x] 1.8a Bound nothing on an ad-hoc `core::ops` bundle. The code being moved does this throughout — `mat_mult_impl` takes `T: Copy + Clone + Mul<Output = T> + Zero + PartialEq + Default`, `transpose_impl` takes `T: Copy + Zero`, `vec_mult_impl` takes `T: Copy + Zero + Add + Mul` — and each is a semiring spelled longhand. Restate them as tower traits, keeping `Copy`/`Clone`/`Default` where the representation needs them
- [x] 1.8b Record every bound loosened from `Field` or `RealField` with the number set it newly admits, so phase 2 can write the test that instantiates it
- [x] 1.9 Confirm the crate declares no scalar trait, marker or newtype of its own
- [x] 1.10 Declare an HKT witness per **element-generic** container — dense matrix, vector, sparse matrix — matching the trait set `CsrMatrixWitness` implements (`HKT`, `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad`, `Adjunction`). `PackedGf2` is excluded structurally: `HKT` projects `Type<T>`, and its element type is fixed to `Gf2` by the packing. Document the exclusion where a reader looks for the witness
- [x] 1.10a Declare the tower impls per container: each matrix reaches `Ring` via `AbelianGroup` + `MulMonoid` + `Distributive` + `Annihilating`, with `Associative<Multiplicative>` and deliberately without `Commutative<Multiplicative>`; the vector reaches `AbelianGroup` with the additive markers only
- [x] 1.10b Declare `Module<R>` for every container, the vector included — the tower's name for a vector space, and the bound that admits ℤ where `Field` would not
- [x] 1.10c Document at each impl site any tower trait a container stops short of, and why
- [x] 1.11 Declare the error types and the conversions among representations
- [x] 1.12 Add `BUILD.bazel`; confirm `cargo build` and `bazel build //deep_causality_linear/...` both succeed
- [x] 1.13 Confirm no algorithm exists yet: every public function body is `todo!()` or a trivial accessor
- [x] 1.14 Confirm the band assignment compiles as intended: `rref` over `i64` must FAIL to compile; `determinant` over `i64` must compile; matrix subtraction over `u64` must FAIL to compile; `rref` over `Gf2` must compile; any operation that halves must FAIL to compile over `Gf2`

## 2. Write the full suite against the unimplemented API

Exit condition: every test compiles, every test fails, and every failure is the unimplemented panic.

- [x] 2.1 Build the test tree mirroring `src` file for file, every module registered upward, every directory declared in `tests/BUILD.bazel`
- [x] 2.2 Put every shared test helper under `src/utils_tests/` — never inside `tests/`, which Bazel cannot reach — and give each helper its own test at `tests/utils_tests/<name>_tests.rs`, since helpers in `src` are library code and count toward coverage
- [x] 2.3 Write one test per scenario in `linear-matrix-representations`, `linear-dense-algorithms`, `linear-f2-algebra` and `linear-crate-identity`; record the scenario → test mapping in a committed `SCENARIOS.md`
- [x] 2.4 Cover the enumerated corner cases: 0×0, 0×n, n×0, 1×1, non-square both ways, zero row, zero column, singular, non-singular with a zero `(0,0)`, rank-deficient, column count not a multiple of the word width, same matrix at two word widths, `{-1,0,1}` reduced mod 2, an entry outside `{0,1}` offered to the packed constructor, an empty CSR row, an out-of-shape index, a near-zero float pivot with a larger one below
- [x] 2.5 Write the Cayley-Menger regression: the 5×5 CM matrix of a regular unit tetrahedron must give `det = 4.0` and `vol = √2⁄12` (`prototype/cm_det.py` reproduces both, and shows an unpivoted elimination returning zero)
- [x] 2.6 Write the 𝔽₂ oracle tests: kernel vectors annihilate and number `cols − rank`; the image basis has `rank` elements and spans the columns; two word widths agree on rank and pivot columns
- [x] 2.7 Write the scalar-band tests: the three compile-fail cases from 1.14 as `trybuild`-style or documented negative tests, and one positive test per band showing the admitted scalars
- [x] 2.8 Write the integer tests: fraction-free determinant equals the exact rational answer with no float appearing; exact rank on a matrix whose singular values straddle `1e-5`; the three ranks disagreeing on one matrix
- [x] 2.9 Write the vector tests: dot against a manual sum, outer product shape, length mismatch rejected, slice round-trip, Hermitian inner product real and non-negative on a complex vector, norms of `[3, -4]` giving 7 / 5 / 4, zero vector giving no `NaN`
- [x] 2.10 Write the solve tests: known system, singular rejected, zero `(0,0)` handled, one factorisation applied to three right-hand sides, triangular substitution against the general path, zero diagonal rejected, wrong triangle rejected
- [x] 2.11 Write the HKT law tests per witness: functor identity and composition, monad left/right identity and associativity, comonad `extend(extract)`, and shape preserved by `fmap`
- [x] 2.11a Pin the tower memberships at compile time in `src/traits/tower_pins.rs`, not as tests: a test calling a function bounded on `Ring` passes by compiling, so running it checks nothing the build has not. The half that is a real check — each matrix **rejected** by `CommutativeRing` and by `IntegralDomain` — is a `compile_fail` doctest, since "must not compile" is not established by the build succeeding
- [x] 2.11b Write one instantiation test per bound recorded in 1.8b, calling the loosened operation at the number set it newly admits — `i64` for every operation moved off `Field`, `u64` for every operation moved to `CommutativeSemiring`
- [x] 2.12 Write the delegation tests: each decomposition's result and error variant, to be compared against `CausalTensor`'s current output in phase 5
- [x] 2.13 Ported, and it is what discharges 4.11. Deferred out of phase 2 deliberately — those tests already existed and passed against code that had not moved, so writing them before the implementation had nothing to add. Run against the reimplementation they found seven divergences
- [x] 2.14 Run the suite: confirm every test fails, and that each failure is the unimplemented panic rather than a compile error, a missing import, or an unrelated panic
- [x] 2.15 Confirm the suite needed no addition to the public surface to compile — if it did, that is an API design finding: record it and revise phase 1

## 3. Verify the suite before trusting it

Exit condition: the suite is shown to reject every known defect class, and the tree is clean again.

**The premise here needed correcting.** As written, these tasks follow phase 2, where every body is
`todo!()` and so every test already fails — injecting a defect changes nothing observable. Showing a
suite *rejects* a defect needs a baseline that *passes* first. Each defect class was therefore run as
implement → verify green → inject → verify red → revert, pulling the phase-4 tasks it needed forward
(4.1, 4.2, 4.5, 4.6, 4.9, 4.10 in part). Those are marked where they landed.

**Three of the ten defects were not caught on the first attempt**, and the suite was strengthened
before they were: 3.5 and 3.6a needed `compile_fail` doctests, because a widened bound and a false
law marker change no runtime value; 3.7 needed a strictly larger margin on a worse-conditioned
system, because the original comparison was satisfied trivially by the defect it was meant to catch.

- [x] 3.1 Stub a deliberately unpivoted elimination; confirm a Cayley-Menger test fails
- [x] 3.2 Stub a packed row update that skips one word, then one that repeats a word; confirm a test fails in each case
- [x] 3.3 Replace an 𝔽₂ exactness check with a tolerance comparison; confirm a test fails
- [x] 3.4 Perturb a reported rank by ±1; confirm a test fails in both directions
- [x] 3.5 Widen a band deliberately — bound `rref` on `Field` where the body needs a pivot rule, or drop `EuclideanDomain` to `CommutativeRing` on the integer determinant; confirm a test or a compile-fail case catches it
- [x] 3.6 Break one HKT law in one witness; confirm a law test fails
- [x] 3.6a Claim `Commutative<Multiplicative>` on a matrix container; confirm a test fails, because the claim is false for matrix multiplication
- [x] 3.6b Remove one container's `Distributive` impl; confirm the `Ring` admission test fails — this is the defect the crate inherits, so the suite must catch it
- [x] 3.7 Replace `solve` with invert-then-multiply; confirm the ill-conditioned residual test fails
- [x] 3.8 Check the scenario → test mapping is complete; every scenario in the four capabilities names at least one test
- [x] 3.9 Check the corner-case list against the suite; every case names a test
- [x] 3.10 Revert every deliberate defect; confirm the tree is clean

## 4. Implement until the suite is green

Exit condition: suite green under both build systems, full coverage, clippy clean.

**Two tasks say "move" and neither did.** `CsrMatrix`, the CG solvers and the six decompositions were
reimplemented. That is a larger claim than a file move and it needs evidence, so each was checked
against the code it replaces: the sparse suite ported and run (2.13), and the decompositions run side
by side with the tensor implementations on the same inputs. Seven divergences in the first, none in
the second.

**Phase 3 pulled 4.1, 4.2, 4.5, 4.6, 4.9 and 4.10 forward**, since a defect injection needs a passing
baseline. They are marked here, where they belong.

**4.14 is met at 98.73%**, against the 95% agreed for this change. Driving it there was worth more than the number: it found that the eigendecomposition's rotation loop and Bareiss's pivot swap had never run under test.

- [x] 4.1 Implement the dense representation and its trait impls, with magnitude pivoting for float scalars
- [x] 4.2 Implement the bit-packed 𝔽₂ representation with whole-word row updates; it overrides no pivot rule
- [x] 4.3 Implement CSR: the read trait only. Do not implement row operations for it — an axpy changes the non-zero pattern — and document that in the module header
- [x] 4.4 Implement the conversions, fallible where the target cannot hold the source's values
- [x] 4.5 Implement `rref`, `rank`, `kernel_basis`, `image_basis`, `solve`, generic over the row-operation trait, naming no concrete representation or scalar
- [x] 4.6 Implement `determinant` over `Field`: pivot by column search always; closed forms at order ≤ 3, elimination at order ≥ 4
- [x] 4.7 Implement the integer path over `EuclideanDomain`: Bareiss fraction-free determinant and exact rank, using `div_euclid` and `normalize`, with no value converted to a float
- [x] 4.8 Implement the vector, its products and the norms; implement matrix–vector for all three representations, sparse without densifying
- [x] 4.9 Implement `solve`, the reusable LU factorisation carrying its permutation, and forward/backward substitution; document on `inverse` that `solve` is preferred for `A⁻¹b`
- [x] 4.10 Implement the HKT witnesses; verify the laws hold at representative values
- [x] 4.10a Implement the tower impls per 1.10a–1.10c; confirm every container is admitted by a `Ring` bound and a `Module<R>` bound, and that no matrix claims `Commutative<Multiplicative>`
- [x] 4.10b Implement the packed 𝔽₂ representation against `Gf2` from `deep_causality_num`; confirm this crate declares no scalar type of its own
- [x] 4.11 **Reimplemented rather than moved**, then checked against the original by porting its suite (2.13). A `git mv` is faithful by construction; a reimplementation is faithful only if measured. Seven divergences found, all invisible when reading the two side by side — the worst a silently reordered CG signature that also inverted the meaning of a `&[R]` parameter. All closed; `openspec/notes/linear/PORTING-FINDINGS.md` carries them
- [x] 4.12 **Reimplemented rather than moved**: one-sided Jacobi for the SVD, modified Gram-Schmidt for QR, cyclic Jacobi for eigen, because the captured baseline showed the existing power iteration converging only to ~1e-8. Both run side by side on the same inputs: they agree at 1e-6 on every case, and the replacement is exact on `diag(1,3)` where the original errs by `1.742e-8`. Table in `DELEGATION-BASELINE.md`
- [x] 4.13 Fix implementation, not tests, wherever the suite disagrees; if a test's assertion is wrong because the API is wrong, change the API and say so
- [x] 4.14 `cargo llvm-cov`: **98.73%**. 30 of 2369 lines missed, 12 of them `traits/tower_pins.rs`, whose `const _: () = {...}` blocks never execute — a justified exception — leaving 18 real lines across the crate. The threshold agreed for this change is 95%. Closing the gap from 95.36% found a genuine defect in the suite rather than padding a number: **`eigen_hermitian`'s rotation loop had never executed**, because every eigen test used a diagonal matrix and a diagonal matrix needs no rotation — the tests would have passed against an implementation that did nothing. Same for Bareiss's pivot swap, which no integer test had reached
- [x] 4.15 Clippy clean with no new `#[allow]`; `cargo test` and `bazel test //deep_causality_linear/...` both green

## 5. Migrate downstream, gated on phase 4

Exit condition: the workspace builds against the new crate and the old name still works.

- [x] 5.1 Confirm phase 4's exit condition holds before touching any consumer — 468 tests, 4 doctests, clippy clean, Bazel 1214/1214, coverage 98.48%
- [x] 5.2 Added as a dependency (no cycle: linear's deps are `num`, `algebra`, `haft` only) and `MatrixView for CausalTensor` implemented in `deep_causality_tensor/src/extensions/ext_linear.rs`, with 8 tests. A tensor of rank ≠ 2 presents as `0 × 1`, not `0 × 0`: the obvious choice is the wrong one, because the determinant of the empty matrix is the empty product and a rank-3 tensor would get a confident `1` back
- [x] 5.3 **Nine of the twelve linear-algebra members now delegate; three stay, on evidence.**

      Delegating: `qr`, `eigen_hermitian`, `sym_eig`, `svd`, `svd_truncated`, `inverse`, `cholesky_decomposition`, `solve_least_squares_cholsky`, and the slice-level Jacobi shared by the DMRG solver.

      **Staying, because they are N-index operations and `linear-crate-identity` forbids those in this crate** ("no operation whose domain is a tensor of rank other than two"):
      - `matmul` builds an einsum AST node (`EinSumOp::MatMul` → `execute_ein_sum`). It is the tensor crate's contraction engine, not a rank-2 product; delegating would replace a general contraction with a triple loop.
      - `norm_l2` and `norm_sq` reduce over `self.data` — the whole flattened buffer at any rank. They coincide with the Frobenius norm only at rank 2.

      **The blocker and how it was closed.** Linear's decompositions were bounded on `RealField`; the tensor's are on `ConjugateScalar`, and `Complex` is not a `RealField` — it is unordered, so no ordered-field bound can cover it. Delegating would have dropped complex support and broken `deep_causality_quantum` (`density_matrix.rs:88`). Closed by moving the kernels rather than widening bounds, which is what `linear-dense-algorithms` said in the first place; phase 4 reimplemented instead (4.12), and that is where the divergence came from.

      `deep_causality_linear/src/algorithms/kernels.rs` now holds the Hermitian-Jacobi eigen, the thin Householder QR and the one-sided Jacobi SVD. `algorithms/cholesky.rs` is new — neither Cholesky nor least squares existed in the crate at all — and both are `ConjugateScalar`-generic, so a Hermitian complex matrix factors as `A = L Lᴴ`.

      **Three defects closed on the way:** linear's SVD returned `cols` singular values where a matrix has `min(m, n)`; the eigen sweep used an absolute ε threshold that never terminates for a large-magnitude matrix; and `LinearError` gained `NotPositiveDefinite`, because a matrix can be invertible and indefinite — `diag(1, −1)` is both — so "no Cholesky factor" and "no inverse" are different failures.

      **Verified.** `qr`/`eigen_hermitian` bit-identical to what they replace (`max|tensor − linear| = 0.000e0`, `Complex<f64>` and `f64`). Cholesky exact against NumPy's `linalg.cholesky`; least squares exactly `[3.5, 1.4]` against `linalg.lstsq`; inverse to 1.1e-16 with `A·A⁻¹ − I` at 2.2e-16; SVD reconstruction ~1e-15 with the identity now **exact** where power iteration reached ~1e-8. Every error variant preserved: `SingularMatrix`, `DimensionMismatch`, `ShapeMismatch` all still returned where they were before.

      **Cost: a `FromPrimitive` bound** on `Tensor::svd`, `inverse`, `cholesky_decomposition` and `solve_least_squares_cholsky` — `ConjugateScalar` and `NormedScalar` both require it. The cascade was measured across the whole workspace and reached exactly two further sites, both in `deep_causality_physics`: `kalman_filter_linear_kernel` and its wrapper. Every scalar in the workspace satisfies it.

      Workspace 1216/1216; linear 496 tests, tensor 544, quantum 171. One warning left: `CausalTensor::get_ref`/`set` are now unused in `src` — my change orphaned them, and AGENTS.md:84 says not to delete unused code unless asked

- [x] 5.4 Recorded before and after on the same machine (M3 Max, 16 cores, 128 GB) by stashing to the pre-delegation tree, benchmarking, and restoring. Only two of the twelve linear-algebra methods have benchmarks at all — `tt_svd_truncated_48x48` and `tt_qr_48x48` — so the other ten have no baseline to diff and would need benchmarks written first.

      | benchmark | before | after | change |
      |---|---|---|---|
      | `tt_svd_truncated_48x48` | 705.38 µs | 693.74 µs | 1.7% faster |
      | `tt_qr_48x48` | 37.477 µs | 37.375 µs | 0.3% faster |

      The first delegation *did* regress QR by 4.7% with non-overlapping confidence intervals — `flatten` read every entry through `MatrixView::get`, a bounds check per entry, where the buffer was already contiguous row-major. Closed by giving `MatrixView` a `to_row_major` hook whose default is the per-entry walk and which `DenseMatrix` and `CausalTensor` override with the copy
- [x] 5.5 Moved with `git mv` so the history follows. The feature is gone everywhere — the `[features]` entry, the optional dependency, the four `#[cfg]` gates, both Bazel feature strings and the example's `features = [...]`.
      Two things the move forced, both verified by compiling rather than reasoned about: the `TryFrom` and `Iso` impls **do** survive it (the orphan rule permits them because `CausalTensor` is local), but the inherent `CsrMatrix::to_dense` does **not** — E0116, a crate cannot write an inherent impl for a foreign type. It is now the `ToDenseTensor` extension trait, which costs the one call site a `use` line.
      The conversion also gained `CommutativeSemiring` on its scalar, because linear's `from_triplets` asks more than sparse's did
- [x] 5.6 `lib.rs` is now re-exports and nothing else. The surface matches item for item **with two stated exceptions**, both cases where matching it would mean re-exporting something demonstrably wrong:
      - `CgFailure` is an enum where it was a struct. Code that destructured it gets a compile error rather than a plausible wrong message — two of the three failure modes are not non-convergence and have no residual to report, which the single struct forced them to claim.
      - `CsrMatrixWitness` does not claim `Monad` or `Adjunction`. Re-measured during this phase: `bind(m, pure)` flattens to `1 x count` and renumbers the columns, so a 1x3 row with a gap comes back with its entry moved from column 2 to column 1. Table in `HKT-LAW-FINDINGS.md`.
      Getting here needed ten `CsrMatrix` methods ported into linear (`compat.rs`) — seven of them reached by live topology code — plus `map_values` widened back to `FnMut` and `from_triplets` loosened off `CommutativeSemiring`, which was an over-bound its body never used.
      `SparseMatrixError` is now an alias for `LinearError`, which absorbed its four failures
- [x] 5.7 Notice at the top of the README: successor named, no further development, a repointing table, and the two changes a re-export cannot hide. `reverted/README.md` records why the old implementation is kept rather than deleted — it is the reference the replacement was checked against, and the `ported_*` suites are it
- [x] 5.8 79 source files repointed across topology, physics and both example crates; `deep_causality_linear` added as a dependency to each. Zero references to the old crate remain outside it.
      Four sites needed more than a changed `use`: the `CgFailure` destructure in `hodge_decomposition_impl.rs` and three field accesses in `leray.rs` and `wall_hodge_star_tests.rs`. Each now reports the failure it actually got — `CgFailure` gained a `Display` impl (neither crate had one), and topology's Hodge path distinguishes a non-positive-definite Laplacian and a wrong-length operator result from non-convergence, which the old struct reported all three as
- [x] 5.9 All retargeted, deduplicating where a target already listed `//deep_causality_linear`. The shim's own `BUILD.bazel` now depends on linear alone, and the moved `tests/BUILD.bazel` is detached under `reverted/`
- [x] 5.10 **The Bazel declaration was spurious.** No file under `deep_causality_cfd` names `deep_causality_sparse` or any symbol of it, and the crate builds and its tests build without it. Two more were stale the same way — `deep_causality_metric` and `deep_causality_multivector`, also zero references — so all three are removed and `//deep_causality_cfd/...` builds green
- [x] 5.11 **Registration is gone, not updated.** `sbom.sh` no longer carries a crate list; `build/scripts/crates.sh` reads the workspace members from the root `Cargo.toml` and every consumer sources it. All 29 crates have an SBOM and its `.sha`, `deep_causality_linear` included.

      The four scripts that carried the list by hand had each drifted, and each was missing a *different* set — `sbom.sh` 28 entries, `miri.sh` 27, `check.sh` 27, `format.sh` 28. Nothing failed when a crate was absent; the loop skipped it, so a crate could ship unformatted, unaudited and without an SBOM with no signal at all. `crates.sh` refuses to continue on an empty list rather than becoming a silent no-op, and `dc_crates_except` warns when an exclusion names a crate that no longer exists. Exclusions moved to the call sites where they can be read: `miri.sh` skips `deep_causality_cfd` with the reason beside it, rather than by omission.

      It proved itself twice within the hour — it flagged `deep_causality_sparse` still declared by `physics` and `topology` after the repoint had made it unused, and it dropped `deep_causality_macros` on its own when that crate was yanked
- [x] 5.12 All six updated. Two things the task did not anticipate:

      **The tier block was already wrong before this change.** `deep_causality_haft` was listed at Tier 0 but has depended on `deep_causality_algebra` at runtime for some time, and `deep_causality_macros` was still listed after being moved to `yanked/`. The block is now generated from the `[dependencies]` tables of each member's `Cargo.toml` — dev- and build-deps excluded — rather than hand-maintained, and a note says so. Tiers run 0–8 where they ran 0–5: `deep_causality_linear` sits at 3, which pushes `tensor` to 4, `multivector` to 5, `topology` to 6, `physics`/`algorithms` to 7 and `cfd`/`discovery` to 8.

      Also corrected while there: `deep_causality_num_rational` and `deep_causality_quantum` were missing from §Project Structure, the scope sentence said 24 crates against 29, the external-dependency count said 21 against 23, `deep_causality_discovery` was missing from `rand`'s dev-dependents, and `deep_causality_tensor`'s description was the single word "Tensors".

      **`uniform-math.md` claimed `Monad`.** Its "Sparse matrices" section advertised `Functor`, `Applicative` and `Monad` on the CSR matrix. The crate does not implement `Monad` and should not — a shaped container cannot satisfy right identity. The section is rewritten for `deep_causality_linear` and states which instances exist and why `Monad` is absent.

      `whats-new-in-deep-causality.md` is left alone: it is a release note describing a past release, and rewriting it would falsify the record
- [x] 5.13 Confirmed by diff. 34 archive files mention the old crate and none was touched: zero overlap with the four archive files that do show as modified — those predate this session and concern `deep_causality_effects` moving to `yanked/` — and none of the 34 mentions `deep_causality_linear`, so no repointing leaked in
- [x] 5.14 All 15 rebuilt and green: 6,043 tests across the 8 in-workspace crates (algorithms 366, cfd 943, discovery 358, multivector 360, physics 1751, quantum 171, tensor 608, topology 1486) and 82 examples building across the 7 example crates.

      **"No source edit" holds with one exception, and it is the documented one.** Of the 15, only `deep_causality_physics` needed an edit caused by the tensor change: `FromPrimitive` added to `kalman_filter_linear_kernel` and its wrapper, because `ConjugateScalar` and `NormedScalar` both require it. Every other edit among the 15 belongs to task 5.8's sparse repoint, not to this one — `topology`'s 67 files, `mathematics_examples`' 9 and `physics_examples`' 1 are all `use` lines and the four `CgFailure` sites
- [x] 5.15 Both green. `cargo test --workspace`: 128 suites, 0 failed. `bazel test //...`: 1206 of 1206 pass

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
