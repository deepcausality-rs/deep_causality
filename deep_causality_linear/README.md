[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# deep_causality_linear

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_linear

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_linear/latest/deep_causality_linear/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

## Summary

Linear algebra for the [DeepCausality project](http://www.deepcausality.com). The crate owns the workspace's matrix
representations and the algorithms over them: sparse (CSR), dense row-major, and bit-packed 𝔽₂ matrices, a dense
vector, eliminations, decompositions, direct and iterative solvers, and an exact integer path. It has zero external
runtime dependencies; its only dependencies are `deep_causality_num`, `deep_causality_algebra`, and
`deep_causality_haft`.

Two decisions shape everything else. Storage is a local choice rather than an architectural one, because all three
representations sit behind one read trait. And every operation is bounded on the weakest trait from the algebra tower
that makes it correct, which is what admits the integers and 𝔽₂ alongside the floats.

The crate defines no scalar trait, marker, or newtype of its own. `Gf2`, `Float106`, and the primitives come from
`deep_causality_num`; the laws they satisfy come from `deep_causality_algebra`.

`CsrMatrix`, its HKT witness, and the conjugate-gradient solvers arrived here from `deep_causality_sparse`, which is
retired and now a re-export shim. Signatures, convergence behaviour, and iteration counts carried over unchanged. Two
things did change: `CgFailure` is a three-case enum where the retired crate had one struct, and the errors of both
crates fold into a single `LinearError`.

## Three representations, one read seam

Choosing a representation is the central decision in linear algebra, so the crate holds all three.

| Type            | Storage                    | Reach for it when                                                  |
|-----------------|----------------------------|--------------------------------------------------------------------|
| `CsrMatrix<T>`  | compressed sparse row      | boundary and coboundary operators, discrete Laplacians, adjacency   |
| `DenseMatrix<T>`| dense row-major            | covariance, metric tensors, density matrices, anything decomposed   |
| `PackedGf2<W>`  | one bit per entry, in `W`  | mod-2 elimination, homology read as a code                          |
| `DenseVector<T>`| contiguous buffer          | chains, states, right-hand sides, coefficient vectors               |

The vector is the larger half of the workload rather than an ornament on the matrix work. A census across the seven
consumer crates counted 60 rank-1 constructions against 46 rank-2.

Three traits form the seam:

| Trait         | What it gives                                            | Implemented by                          |
|---------------|----------------------------------------------------------|-----------------------------------------|
| `MatrixView`  | shape and entries by value                                | all three matrices, plus `CausalTensor` |
| `MatrixBuild` | `zeros`, `identity`, `set`                                | all three matrices                      |
| `RowOps`      | `swap_rows`, `scale_row`, `axpy_rows`, `pivot_in_column`  | dense and packed 𝔽₂ only                |

`get` returns `Self::Scalar` by value, because a bit-packed matrix has one bit inside a word and no element to lend a
reference to. Reading a position outside a sparse matrix's stored pattern returns the scalar zero and is not an error;
only an index outside the *shape* fails.

`CsrMatrix` implements the read side and stops there. Adding a multiple of one sparse row to another changes that row's
non-zero pattern, which in CSR means reallocating every row after it. Sparse elimination wants a fill-reducing ordering
and a symbolic factorisation, so a caller who needs it converts to a dense layout and writes that conversion at the call
site, where its cost is visible.

```rust
use deep_causality_linear::{CsrMatrix, MatrixView, PackedGf2, csr_to_dense};

// Sparse: only the stored entries cost anything.
let sparse = CsrMatrix::from_triplets(3, 3, &[(0, 0, 2.0), (1, 1, 3.0), (2, 0, 1.0)]).unwrap();
assert_eq!(sparse.shape(), (3, 3));
assert_eq!(sparse.values().len(), 3);

// A position outside the stored pattern is a zero.
assert_eq!(sparse.get(0, 2).unwrap(), 0.0);

// Dense: the layout the eliminations and decompositions work on.
let dense = csr_to_dense(&sparse);
assert_eq!(dense.shape(), (3, 3));

// Bit-packed 𝔽₂: entries reduce mod 2 on the way in, so {-1, 0, 1} is accepted.
let packed = PackedGf2::<u64>::from_i64_mod2(&[1, -1, 0, 0, 1, 1], 2, 3).unwrap();
assert_eq!(packed.shape(), (2, 3));
```

`PackedGf2` is generic over its word type. A caller picks the width that suits the target, and the suite runs at a
narrow width so that a column count crossing a word boundary shows up in a matrix small enough to read. The same matrix
packed at two widths reports the same rank and the same pivot columns, and that is a test rather than an assumption.

## Where this sits against `deep_causality_tensor`

The split is by arity, not by density. A two-index object is a matrix and lives here. `ein_sum`, broadcasting, the
Kronecker product, the axis reductions, and the tensor-train stack take an N-index object and stay in the tensor crate.

`deep_causality_linear` sits **below** `deep_causality_tensor` in the dependency graph and never depends on it.
`CausalTensor`'s rank-2 decompositions delegate into the bodies here, which is why a density matrix in the quantum layer
and a stiffness matrix in the fluids layer run the same kernel.

## Bounded by algebra

Each operation names the algebra rung it actually needs:

| Operation                           | Bound              | Admits                            |
|-------------------------------------|--------------------|-----------------------------------|
| transpose, dot product              | `CommutativeSemiring` | ℕ upward                       |
| entrywise subtraction, `Module<R>`  | `CommutativeRing`  | ℤ upward                          |
| `determinant_exact`, `rank_exact`   | `EuclideanDomain`  | ℤ only                            |
| `rref`, `rank`, kernel and image    | `Field`            | 𝔽₂, ℚ, ℝ, ℂ                       |
| `rref_stable`, `determinant`, `solve` | `NormedScalar`   | ℝ, ℂ, `Float106`                  |
| the decompositions, Cholesky        | `ConjugateScalar`  | ℝ, ℂ, `Dual` (forward-mode AD)    |
| the norms                           | `NormedScalar`     | ℝ, ℂ, `Float106`                  |

The determinant is a polynomial in the entries and needs no division, so it is defined over any commutative ring.
Gaussian elimination divides by its pivot and leaves ℤ on the first step. Both facts are in the bounds.

Four `compile_fail` doctests hold the line where a widened bound would still compile and still pass every behavioural
test: `f64` refused by `EuclideanDomain` on both integer entry points, and a matrix refused by `CommutativeRing` and by
`IntegralDomain`. `[[1,0],[0,0]]` times `[[0,0],[0,1]]` is zero with neither factor zero, so cancellation fails over
matrices, and Bareiss elimination rests on cancellation.

The container memberships run the other way. Every matrix reaches `Ring` and `Module<R>`; the vector reaches
`AbelianGroup` and `Module<R>` and no multiplicative rung, having no `Mul` that returns a vector. `Module<R: Ring>` is
the tower's name for a vector space, and stating it over a *ring* is what admits `DenseVector<i64>`, which topology's
integer chains need. These memberships are pinned in `src/traits/tower_pins.rs` as ordinary items rather than as tests,
because a membership test passes by compiling and reports nothing the build has not already settled.

## Elimination, and everything read off it

Every entry point runs the same private elimination over the `RowOps` seam, naming no representation, no scalar, and no
word width. They come in pairs, because the pivot rule cannot be chosen by the representation alone:

| Suffix    | Pivot                                | Admits                                       |
|-----------|--------------------------------------|----------------------------------------------|
| none      | first non-zero at or below the row    | any `Field`: 𝔽₂, ℚ, ℝ, ℂ                     |
| `_stable` | largest modulus at or below the row   | any `NormedScalar`: ℝ, ℂ, `Float106`         |

The exact rule needs no ordering and no epsilon, which is how 𝔽₂ and ℚ get through. Over the floats a pivot near zero
amplifies rounding, so a float caller wants the `_stable` entry point. Both search the column; neither takes the
diagonal on faith. That matters more than it sounds: a Cayley-Menger matrix has `m[0][0] = 0` by construction, and an
elimination that assumes the diagonal returns zero for every simplex volume.

`rref` and `rref_stable` return a `Reduced` carrying the rank and the pivot columns, since both come out of one pass.
`rank`, `rank_stable`, `kernel_basis`, `image_basis`, and `determinant` read off the same core.

```rust
use deep_causality_linear::{DenseMatrix, DenseVector, determinant, rank_stable, solve};

let a = DenseMatrix::from_vec(vec![4.0, 1.0, 1.0, 3.0], 2, 2).unwrap();

assert_eq!(determinant(&a).unwrap(), 11.0);
assert_eq!(rank_stable(&a).unwrap(), 2);

let b = DenseVector::from_vec(vec![11.0, 0.0]);
let x = solve(&a, &b).unwrap();
assert_eq!(x.as_slice(), &[3.0, -1.0]);
```

## Solving

| Entry point           | Method                                                                 |
|-----------------------|------------------------------------------------------------------------|
| `Lu::factor`          | LU with partial pivoting, kept as a value so it can be applied often    |
| `solve`               | one factorisation, one application                                     |
| `solve_lower` / `solve_upper` | forward and back substitution, no factorisation               |
| `inverse`             | one factorisation, `n` applications                                    |
| `cholesky`            | `A = L Lᴴ` for a Hermitian positive-definite `A`                       |
| `solve_least_squares` | the normal equations through the Cholesky factor                       |

`Lu` is a value because factorising costs `O(n³)` and each application costs `O(n²)`. Both workloads in this workspace
that solve repeatedly, the Kalman filter in `deep_causality_physics` and the ridge fits in `deep_causality_algorithms`,
hold one matrix and many right-hand sides. The permutation travels inside the factorisation, since applying `L` and `U`
without it solves a different system.

Positive-definiteness is discovered rather than asserted. The Cholesky factorisation *is* the test, and the first
diagonal entry whose radicand is non-positive is where the input is shown not to qualify;
`LinearError::NotPositiveDefinite` carries that index. A matrix can be non-singular and still fail there, which is why
it is a separate variant from `Singular`.

## Decompositions

`svd`, `svd_sorted`, `svd_truncated`, `singular_values`, `qr`, `eigen_hermitian`, and `cholesky` are bounded on
`ConjugateScalar`. That bound spans real fields, dual numbers for forward-mode AD, and complex. Magnitudes and
thresholds live in the associated real type and only the rotations are injected back, so a Hermitian complex matrix
decomposes as readily as a real symmetric one, and `RealField` could never cover it because `Complex` is unordered.

`qr` and `eigen_hermitian` take `MatrixView` rather than `RowOps`. They copy the entries into a flat buffer and never
mutate a row, so demanding the mutating trait would exclude every read-only representation for nothing.

Thresholds scale by the input's Frobenius norm. An absolute epsilon burns the whole sweep budget on a large-magnitude
matrix and fires immediately on a small one.

```rust
use deep_causality_linear::{DenseMatrix, cholesky, eigen_hermitian, qr, MatrixView};

let a = DenseMatrix::from_vec(vec![4.0, 1.0, 1.0, 3.0], 2, 2).unwrap();

let l = cholesky(&a).unwrap();          // A = L Lᴴ, lower triangular
assert_eq!(l.as_slice()[0], 2.0);
assert_eq!(l.as_slice()[1], 0.0);

let (q, r) = qr(&a).unwrap();           // thin: Q is m×k, R is k×n, k = min(m, n)
assert_eq!(q.shape(), (2, 2));

let (values, vectors) = eigen_hermitian(&a).unwrap();   // A = V diag(λ) Vᴴ
assert_eq!(values.len(), 2);
assert_eq!(vectors.shape(), (2, 2));
```

`Truncation` keeps "at most rank k" and "everything above epsilon" as distinct requests, so a caller who means one does
not silently get the other.

## Exact paths

Rank over ℝ, rank over ℤ, and rank over 𝔽₂ are three different questions. Each has its own entry point, so the choice
of field is visible at the call site.

### Over 𝔽₂

`rank_gf2`, `kernel_basis_gf2`, and `image_basis_gf2` are the generic elimination fixed to the bit-packed
representation. They take no tolerance and apply none; every non-zero element of 𝔽₂ is its own inverse, so the
elimination divides by nothing that could be near zero.

```rust
use deep_causality_linear::{PackedGf2, kernel_basis_gf2, rank_gf2};

// Rows 0 and 1 sum to row 2 over 𝔽₂, so the rank is two and the kernel is one-dimensional.
let m = PackedGf2::<u64>::from_i64_mod2(&[1, 1, 0, 0, 1, 1, 1, 0, 1], 3, 3).unwrap();

assert_eq!(rank_gf2(&m).unwrap(), 2);
assert_eq!(kernel_basis_gf2(&m).unwrap().shape(), (3, 1));
```

### Over ℤ

`determinant_exact` and `rank_exact` use fraction-free Bareiss elimination. No float appears at any point, including
intermediates. The determinant of an integer matrix is an integer, and Bareiss keeps every intermediate in the ring
while reaching the answer in cubic time, where Laplace expansion takes factorial time.

```rust
use deep_causality_linear::{DenseMatrix, determinant_exact, rank_exact};

let m = DenseMatrix::from_vec(vec![2i64, 3, 1, 4], 2, 2).unwrap();
assert_eq!(determinant_exact(&m).unwrap(), 5);
assert_eq!(rank_exact(&m).unwrap(), 2);
```

`EuclideanDomain` supplies the division; `IntegralDomain` one rung below is what makes those divisions exact, because an
integral domain has no zero divisors and therefore licenses cancellation.

## Conjugate gradient, matrix-free

`cg_solve`, `cg_solve_preconditioned`, and `cg_solve_preconditioned_from` take a closure applying the operator rather
than a matrix, so a caller with a Laplacian it never assembles can use them, and the sparse-versus-dense question does
not arise. `tolerance` is relative to `‖b‖`. `CgFailure` names three cases: a reached iteration limit, an operator that
is not positive definite, and a length mismatch. Each calls for a different response, so each gets a variant.

```rust
use deep_causality_linear::cg_solve;

// A 1-D Dirichlet Laplacian, applied without assembling it.
let apply = |x: &[f64]| -> Vec<f64> {
    (0..x.len())
        .map(|i| {
            let left = if i == 0 { 0.0 } else { x[i - 1] };
            let right = if i + 1 == x.len() { 0.0 } else { x[i + 1] };
            2.0 * x[i] - left - right
        })
        .collect()
};

let x = cg_solve(apply, &[1.0, 1.0, 1.0], 1e-12, 100).unwrap();
assert!((x[0] - 1.5).abs() < 1e-10);
assert!((x[1] - 2.0).abs() < 1e-10);
```

LU on a sparse matrix fills in, and the factors are dense even when the matrix is not. These cover the symmetric
positive-definite case, which is the one the workspace actually solves.

## Vectors and norms

`vector_norm_l1`, `vector_norm_l2`, `vector_norm_sq`, and `vector_norm_inf` are generic over a **slice**, and the
`DenseVector` methods of the same names delegate to them. The slice form exists because
`deep_causality_multivector` holds a `Vec<T>` of coefficients and `CausalMultiField` holds a tensor's buffer; routing
either through a vector type would copy the whole coefficient vector on every norm.

`matrix_norm_l1`, `matrix_norm_inf`, and `matrix_norm_frobenius` are generic over any `MatrixView`, so they apply to the
sparse and packed representations too.

```rust
use deep_causality_linear::{DenseVector, MatrixView};

let v = DenseVector::from_vec(vec![3.0, 4.0]);
assert_eq!(v.norm_l2(), 5.0);
assert_eq!(v.norm_l1(), 7.0);
assert_eq!(v.dot(&v).unwrap(), 25.0);

let outer = v.outer(&DenseVector::from_vec(vec![1.0, 0.0]));
assert_eq!(outer.shape(), (2, 2));
```

`dot` is bounded on `CommutativeSemiring` and is available over ℕ. `hermitian_inner` is separate and computes
`Σ conj(aᵢ) · bᵢ`. Over ℂ the plain dot product induces no norm, and `deep_causality_quantum` works in `Complex<R>`
throughout.

## Higher-kinded witnesses

Three witnesses lift the containers into the `deep_causality_haft` surface, so `fmap`, `fold`, and the rest read the
same here as on a tensor, a manifold, or a propagating effect.

| Witness              | Container         | Traits                                              |
|----------------------|-------------------|-----------------------------------------------------|
| `CsrMatrixWitness`   | `CsrMatrix<T>`    | `Functor`, `Foldable`, `Pure`, `Applicative`, `CoMonad` |
| `DenseMatrixWitness` | `DenseMatrix<T>`  | `Functor`, `Foldable`, `Pure`, `Applicative`, `CoMonad` |
| `DenseVectorWitness` | `DenseVector<T>`  | the same, plus `Monad`                              |

`DenseVector` claims `Monad`; the two matrices stop at `Applicative` and `CoMonad`. A shaped container cannot satisfy
the monad laws: `pure` has to choose a shape for a single value, and right identity `bind(m, pure) == m` then asks
`bind` to reassemble an `m × n` matrix from `m · n` one-by-ones. A vector's only shape is its length, so it satisfies
the laws and states them.

`CsrMatrixWitness::fmap` maps the **stored** entries and leaves the structural zeros alone, which keeps the result
sparse. A caller who wants a function applied to the whole logical matrix densifies first, explicitly.

```rust
use deep_causality_haft::{Foldable, Functor};
use deep_causality_linear::{CsrMatrix, CsrMatrixWitness, DenseVector, DenseVectorWitness};

let v = DenseVector::from_vec(vec![1, 2, 3]);
assert_eq!(DenseVectorWitness::fmap(v, |x| x * 2).as_slice(), &[2, 4, 6]);

// The fold visits the stored entries; a structural zero contributes nothing.
let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1), (1, 1, 2)]).unwrap();
assert_eq!(CsrMatrixWitness::fold(m, 0, |acc, x| acc + x), 3);
```

`PackedGf2` has no witness. It is generic in its *word* and fixed to `Gf2` in its element, so there is no `PackedGf2<T>`
for `Type<T>` to name; the route out is `packed_to_dense_gf2`.

## Conversions

Conversions are explicit and never implicit, because a conversion changes the cost model of everything done afterwards.

| From              | To            | |
|-------------------|---------------|---|
| sparse            | dense         | total; costs memory |
| dense             | sparse        | total |
| dense or sparse   | packed 𝔽₂     | **fallible**: the target holds only `{0, 1}` |
| packed 𝔽₂         | dense `Gf2`   | total |

Only the packing direction fails, and for one reason: an entry outside `{0, 1}`. The error names the position, so a
caller does not re-scan to find it. `csr_to_packed_gf2_mod2` reduces instead, which is what topology's boundary
operators need, their entries being `{-1, 0, 1}`.

## Errors

One error type, `LinearError`, across all representations and all algorithms. It is a newtype over `LinearErrorEnum`,
so a new failure mode is a new variant on the inner enum and a downstream `match` with a wildcard arm keeps compiling.
Variants carry the numbers needed to say what went wrong: `IndexOutOfBounds` carries the position and the shape it was
checked against, `Singular` carries the column elimination stopped at, `WrongTriangle` carries the first offending
position. `CgFailure` is separate, being generic over the real type it reports a residual in.

## Measurements

Four figures from the design work, each behind a decision in the source:

| Measurement                                                        | Result                                                          |
|--------------------------------------------------------------------|-----------------------------------------------------------------|
| Packed `u64` elimination against a byte-per-entry `Gf2` scalar      | 1.7× faster at n=128, 3.2× at n=2048, on one eighth the memory   |
| The `RowOps` seam against a hand-written non-generic loop           | 0.92–0.95× the time, because `from_col` skips columns already zeroed |
| `csr_to_dense` by scatter against reading every position back       | 0.24 ms against 23.6 ms, on 800×800 holding 64000 entries        |
| `to_row_major` as a memcpy against the per-entry default            | the default ran 4.7% slower on a 48×48 QR                       |

Measured on an M3 Max, 16 cores, 128 GB, macOS 26.

## Verification

580 tests cover the crate, laid out to mirror `src`, plus five doctests of which four are `compile_fail` guards.
`SCENARIOS.md` maps every scenario in the four capabilities to the test that serves it, and marks the ones covered by a
compile-time pin instead.

The 𝔽₂ layer is machine-checked in Lean 4 against Mathlib. Four theorems, zero `sorry`, each with a Rust witness test
under `tests/formalization_lean/`. The load-bearing one is rank-nullity: `ChainComplex::betti_number_over` in
`deep_causality_topology` substitutes `n_k − rank ∂_k` for `dim ker ∂_k`, and nothing in either crate's source states
that identity. The witnesses compute the two sides by different routines, the rank by elimination and the nullity by
counting kernel-basis columns, so the test is two independent computations agreeing rather than one rearranged. See
[`LEAN_LINEAR.md`](LEAN_LINEAR.md).

## Dependency

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_linear = "0.1"
```

## Features and `no-std`

| Feature   | Effect                                                            |
|-----------|-------------------------------------------------------------------|
| `std`     | default; enables `std` on the three internal dependencies         |
| `alloc`   | the collections the containers are built from                     |
| `no-std`  | `alloc` plus `no-std` on the three internal dependencies          |

The crate is `#![no_std]` when `std` is off. It always needs an allocator, since every container owns a `Vec`.

```bash
cargo build --no-default-features --features no-std -p deep_causality_linear
cargo test  --no-default-features --features no-std -p deep_causality_linear
```

579 of the 580 tests run in that configuration; the one left out pins `LinearError` as a `std::error::Error`, which
exists only under `std`.

Under Bazel:

```bash
bazel build //deep_causality_linear/...
bazel test  //deep_causality_linear/...
```

## Safety

No `unsafe` — the crate opts into the workspace-wide `unsafe_code = "forbid"` lint policy. No macros in library code.

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under
the [MIT license](https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
