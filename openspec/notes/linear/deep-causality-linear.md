<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `deep_causality_linear` — impact research

Status as of 2026-08-23. This note records the measurements taken before proposing a linear-algebra
crate: where linear algebra lives in the workspace today, what is duplicated, what is wrong, what a
consolidation costs, and what the migration looks like. It is the input to the change
`openspec/changes/add-linear-algebra-crate/`.

## Why this came up

The quantum gap register asks for 𝔽₂ linear algebra. `openspec/notes/quantum/qcl-gaps.md` G-01 —
severity S1, blocking three requirements — records that no mod-2 elimination exists anywhere in the
workspace, and assigns the work to `deep_causality_topology` "because that is where chain complexes
live and topology must not learn about codes."

Placing it there raises the prior question of where linear algebra belongs at all. The answer turned
out to be: in three places, none of them a linear-algebra crate.

## Where linear algebra lives today

### Five matrix representations

| Representation | Defined in | Kind |
|---|---|---|
| `CausalTensor<T>` — `{data, shape, strides}` | `deep_causality_tensor` | strided N-d; rank-2 is a dense matrix |
| `CsrMatrix<T>` | `deep_causality_sparse` | compressed sparse row |
| `Matrix3<F> = [[F; 3]; 3]` | `deep_causality_num/src/alias/mod.rs:9` | fixed 3×3, used only by `num_complex` quaternions |
| `&[Vec<R>]` | `deep_causality_topology/src/types/regge_geometry/curvature.rs:275` | ad-hoc rows-of-rows |
| `&mut [T]` + `n` | `deep_causality_topology/src/types/simplicial_complex/lazy_hodge_star.rs:97` | ad-hoc flat slice |

`AdjacencyMatrix<T>`, `IncidenceMatrix<T>` and `LaplacianMatrix<T>` in
`deep_causality_topology/src/alias/mod.rs` are all aliases of `CausalTensor<T>`; `AbcdMatrix<R>` in
physics and `DensityMatrix<R>` in quantum are newtypes over it. There is no general `Matrix` type.

### The tensor crate is two libraries

`deep_causality_tensor` is 12,334 lines of `src`. Measured by directory:

| | lines |
|---|---|
| 2-D matrix operations — `svd` 117, `svd_decomp` 170, `svd_truncated` 375, `qr` 145, `eigen` 158, `inverse` 123 | **1,088** |
| N-d tensor operations — `ein_sum` 1,310, `reduction` 174, `broadcast` 134, `shape` 120, `view` 84, `qtt` 73, `stack` 61, `kronecker` 58, `product` 55 | 2,069 |
| tensor-train / MPS / MPO (`causal_tensor_network/`) | 3,881 |

The 1,088 lines are dense linear algebra that reached a tensor crate because `CausalTensor` was the
only dense container available. `matmul` sits in `tensor_product/` and is the most-called of the
group — 13 call sites in the physics Kalman filter alone
(`deep_causality_physics/src/kernels/dynamics/estimation.rs`), plus GRMHD, quantum channels and
projections, and multivector conversions.

### Topology carries its own copies

Three determinants over three representations, in one crate:

| Function | Representation | Complexity |
|---|---|---|
| `regge_geometry/curvature.rs:275` `det_recursive` | `&[Vec<R>]` | Laplace expansion, O(n!) |
| `manifold/geometry/mod.rs:145` `determinant_impl` | `&CausalTensor<T>` | Laplace expansion, O(n!), allocates a sub-tensor per cofactor |
| `simplicial_complex/lazy_hodge_star.rs:97` `gaussian_determinant` | `&mut [T]`, `n` | Gaussian elimination, O(n³) |

And two ranks that are near-identical copies. `chain_complex_impl.rs:94` says so in its own doc
comment — "Mirrors the helper used by `CellComplex::rank_of_matrix`". Both densify a
`CsrMatrix<i8>` into `Vec<f64>`, build a `CausalTensor`, call `svd()`, and count singular values
above `1e-5`; roughly 30 lines each, differing in whitespace and comments.

### Physics carries its own small-matrix linear algebra

`deep_causality_physics` is 23,352 lines of `src` and 26,173 of tests — the second-largest consumer
after topology. It reaches linear algebra three separate ways.

**Through `CsrMatrix`, read-only.** Two import sites, `kernels/mhd/ideal.rs:11` and
`kernels/mhd/grmhd.rs:11`, both for matrix–vector products: `apply_csr_real` against Hodge-star
operators carrying manifold scalars, and `apply_csr_i8` against coboundary operators whose entries
are pure ±1. Nothing mutates a CSR matrix and nothing eliminates on one, so the constraint that
sparse implements the read side only costs this crate nothing.

**Through `CausalTensor`.** 18 `matmul` calls — 13 in the Kalman filter
(`kernels/dynamics/estimation.rs`), 4 in GRMHD, 1 in photonics ray transfer — and 2 `inverse()`
calls, at `estimation.rs:158` for the innovation covariance and `grmhd.rs:223` for the metric. That
is the whole of this crate's exposure to the decomposition relocation.

**Hand-rolled, bypassing both.** Fixed-size closed forms that never touch `CausalTensor::inverse`:

| helper | shape |
|---|---|
| `theories/general_relativity/gr_utils.rs:12` `invert_4x4` | `&CausalTensor<T>` in, `[T; 16]` out, cofactor/adjugate |
| `theories/general_relativity/gr_utils.rs:114` `invert_3x3` | `[[T; 3]; 3]` |
| `theories/general_relativity/adm_state.rs:126` `inverse_spatial_metric` | `[[S; 3]; 3]` |
| `kernels/fluids/coherent_structures.rs:211` `symmetric_3x3_eigenvalues` | `[[R; 3]; 3]` |
| `kernels/fluids/kinematics.rs:125` | inline 3×3 determinant by cofactor |

These are correct as written — at n ≤ 4 a closed form beats a general routine and avoids pivoting
round-off entirely — and they are evidence for the small-n rule the change adopts rather than a
target for it. They are out of scope; consolidating them is a separate decision with its own
numerical risk.

**Physics does not compute homology.** It imports `ChainComplex` for `num_cells(grade)` only.
`betti_number` is called nowhere outside `deep_causality_topology`'s own tests, so the exact-𝔽₂
change reaches no consumer.

## The correctness defect this exposes

Those two rank helpers compute homology. `betti_number` is `dim_ker − rank_{k+1}` where
`dim_ker = n_k − rank_k`, so every Betti number the crate reports comes from thresholding
floating-point singular values of a matrix whose entries are `{-1, 0, 1}`.

`qcl-gaps.md` G-02 records the consequence: rank over ℝ is not rank over 𝔽₂. The two agree for the
toric code, which is why nothing has failed yet. A complex with even-weight dependencies has a
smaller 𝔽₂ rank, and the reported `k` would be wrong with no error raised.

An exact integer or 𝔽₂ elimination answers this with no tolerance at all. That makes 𝔽₂ elimination
a correctness fix for topology, not only a new capability for quantum.

## Measurements

### The access seam preserves word-parallel XOR

A prototype in `openspec/notes/linear/prototype/` runs one generic `rref`, written against
a row-operation trait, over four representations of the same 𝔽₂ matrix. M3 Max, 16 cores, 128 GB;
`--release`.

| n | packed `u64`, generic | packed, hand-written | `Dense<Gf2>` (1 byte/bit) | `Vec<Vec<Gf2>>` | packed vs byte | seam cost | memory |
|---|---|---|---|---|---|---|---|
| 128 | 103.25 µs | 110.50 µs | 170.58 µs | 283.25 µs | 1.7× | 0.93× | 2 vs 16 KiB |
| 256 | 410.50 µs | 447.79 µs | 702.50 µs | 1.173 ms | 1.7× | 0.92× | 8 vs 64 KiB |
| 512 | 1.803 ms | 1.896 ms | 3.341 ms | 5.261 ms | 1.9× | 0.95× | 32 vs 256 KiB |
| 1024 | 7.869 ms | 8.494 ms | 18.685 ms | 26.682 ms | 2.4× | 0.93× | 128 vs 1024 KiB |
| 2048 | 34.748 ms | 37.311 ms | 112.517 ms | 148.227 ms | 3.2× | 0.93× | 512 vs 4096 KiB |

Two results. The `Vec<Vec<Gf2>>` column varies by up to 1.5× between runs on the same input and
nothing below rests on it; the other three columns reproduce. `prototype/README.md` records how to
re-run this and what the harness does not control.

**The seam costs nothing.** The generic algorithm through the trait runs at 0.92–0.95× the
hand-written non-generic elimination over `&mut [u64]` at every size — it is slightly faster,
because the trait's `from_col` argument lets the implementation skip the eliminated prefix that the
hand-written loop re-reads. G-01's "roughly 200 lines over `u64`" and a generic algorithm behind a
row-operation trait cost the same to run.

**Bit-packing earns its complexity.** Against the best alternative that the algebra tower alone
would give — a `Gf2` scalar satisfying `Field`, stored one byte per bit — packing is 1.7× faster at
n=128 rising to 3.2× at n=2048, on 8× less memory. The ratio grows with n because the cache pressure
does. This confirms G-01's judgement that 𝔽₂ belongs in packed bitsets rather than as a tower scalar,
and now with a number attached.

### Two constraints the prototype found

**Only two crates can broker `CausalTensor`.** `impl MatrixView for CausalTensor<f64>` is legal in
`deep_causality_linear` (trait is local) or in `deep_causality_tensor` (type is local), and nowhere
else — `prototype/tensor_impl/` compiles the third case and gets E0117. The two legal placements force
opposite dependency edges and cannot coexist, so the orphan rule narrows the field without choosing.
The choice comes from the decompositions: `CausalTensor::svd` must call into linear, so the edge is
tensor → linear, and the impl therefore lives in `deep_causality_tensor`.

A newtype wrapper is the workaround a third crate has, and it costs a wrapper at every call site
while giving `CausalTensor` none of the methods.

**Sparse elimination is a different algorithm.** `swap_rows` is fine on CSR. `axpy_rows` is not:
adding a multiple of one sparse row to another changes that row's non-zero pattern, which in CSR
means reallocating every row after it. Sparse elimination needs a fill-reducing ordering and a
symbolic factorisation. One generic elimination can serve dense and bit-packed, which share a
dense layout. It cannot also serve sparse.

That bounds the claim. `deep_causality_linear` holding sparse and dense side by side means one crate
owning both representations and the algorithms appropriate to each — the shape LAPACK and its sparse
counterparts have — rather than one algorithm covering everything.

## Does a dense matrix type have real call sites?

Yes — 46, and three crates never construct anything else. Measured by taking the rank of every
constructed shape across the seven consumer crates.

| crate | constructions | ranks | 2-D ops called | N-d ops called |
|---|---|---|---|---|
| `deep_causality_topology` | 46 | rank1=26, rank2=20, **no rank≥3** | 23 | **0** |
| `deep_causality_physics` | 20 | rank1=13, rank2=6, rank4=1 | 20 | **0** |
| `deep_causality_quantum` | 10 via `from_slice` | **all rank-2**, 8 square | 13 | **0** |
| `deep_causality_discovery` | 6 | rank1=1, rank2=5 | 0 | 0 |
| `deep_causality_cfd` | 30 | rank1=18, rank2=7, rank3=4, rank4=1 | 1 | 6 |
| `deep_causality_algorithms` | 16 | rank2=8, rank3=5, rank4=1 | 0 | 19 |
| `deep_causality_multivector` | 0 direct | — | 5 | 18 |

118 constructions in total: **60 rank-1, 46 rank-2, 12 rank ≥ 3**. N-d operations means `ein_sum`,
`broadcast_*`, `kronecker`, axis reductions, `reshape`, `stack`, `view`, or the tensor-train entry
points; 2-D operations means `matmul`, `transpose`, `inverse`, `svd`, `qr`, `eigen_hermitian`,
`dagger`.

**Rank ≥ 3 is 10% of constructions and lives in three files**:
`physics/src/theories/electromagnetism/gauge_em_ops_impl.rs:82` (`vec![num_points, dim, dim, 1]`),
`cfd/src/tensor_bridge/operators.rs:36` (`vec![rl, 2, 2, rr]`, an MPO core), and SURD's joint
distributions in `algorithms`.

**Physics, quantum and topology call 56 two-dimensional operations and zero N-d operations between
them.** Quantum's ten shapes are `[d,d]`, `[d_out,d_out]`, `[dim,dim]`, `[d_keep,d_keep]`,
`[d_full,d_full]` and one `[d_out,d_in]`.

**What the type would buy, concretely.** `DensityMatrix` stores `dim: usize` beside its tensor
(`quantum/src/types/density_matrix.rs:29-32`) because `CausalTensor` cannot express squareness. That
invariant is maintained by hand today, as are topology's `AdjacencyMatrix`/`IncidenceMatrix`/
`LaplacianMatrix` aliases. A dense matrix type moves rank and squareness from runtime checks into the
type.

**The qualifier.** 60 of the 118 constructions are rank-1 vectors — more than the matrices — and a
dense *matrix* type serves none of them. The split is 46 matrix sites, 60 vector sites, 12 genuine
tensors.

## Do physics' five small-matrix helpers consolidate?

Partly: one genuine pair, and the fix does not need this crate.

`gr_utils.rs:114` `invert_3x3` and `adm_state.rs:126` `inverse_spatial_metric` are the same function.
The determinant expression and all nine adjugate terms match term for term. They differ in input
shape (`[[T;3];3]` against a flat 9-slice) and in two things that look accidental:

| | `invert_3x3` | `inverse_spatial_metric` |
|---|---|---|
| singularity threshold | `1e-14` | `1e-12` |
| compared in | `T` | `f64`, after `det.into()` |
| error text | "Singular spatial metric (det ~ 0)" | "Spatial metric determinant is zero" |

The same physical quantity — the spatial metric determinant — has thresholds 100× apart depending on
which helper is reached, and the `adm_state` path truncates to `f64`, discarding precision whenever
the scalar is wider (`Float106`). No test pins either threshold; the `1e-12` occurrences under
`deep_causality_physics/tests/` are unrelated propulsion and nuclear assertions.

The 3×3 cofactor determinant appears four times: `gr_utils.rs:118`, `adm_state.rs:144`,
`kinematics.rs:126`, and again as `det_b` inside `symmetric_3x3_eigenvalues`.

`invert_4x4` stays: it extracts a **strided** 4×4 block from a larger tensor (`cols` from
`shape.last()`, reading a 4×6 connection), which is a different operation from inverting a 4×4.
`symmetric_3x3_eigenvalues` stays: Smith (1961) closed form with a diagonal fast path and `acos`
clamping, more accurate here than a general eigensolver.

**Both crates involved are `deep_causality_physics`.** Merging them is a `pub(crate) fn` inside that
crate, so this finding supports a small physics fix and not the linear crate. It is recorded here
because the question was asked, and it is out of scope for this change.

## Do topology's determinants change numerically?

Researched, because the answer decides whether the consolidation is safe.

**What the three determinants are actually fed.** Two of them get Cayley-Menger matrices:

| call site | matrix | order |
|---|---|---|
| `regge_geometry/curvature.rs:254` → `det_recursive` | Cayley-Menger, hard-coded literal | 5×5 (tetrahedron) |
| `manifold/geometry/mod.rs:72` → `determinant_impl` | Cayley-Menger, `matrix_dim = k + 2` | 3×3 up to (dim+2)² |
| `simplicial_complex/lazy_hodge_star.rs:81` → `gaussian_determinant` | Gram matrix, `vectors[i]·vectors[j]` | `k − 1` |

A Cayley-Menger matrix has **`m[0][0] = 0` by construction** — `mod.rs:41` allocates zeros and then
writes `one` only into indices `1..matrix_dim`. A Gram matrix has a strictly positive diagonal.

**The consequence.** `gaussian_determinant` performs no row pivoting and bails on a small leading
pivot (`lazy_hodge_star.rs:104`: `if mat[pivot].abs() < pivot_threshold { return T::zero(); }`).
Consolidating the two Laplace determinants onto it would return **zero for every simplex volume**.
Measured on a regular unit tetrahedron:

| method | det(CM₅ₓ₅) | vol² | vol |
|---|---|---|---|
| Laplace — what `det_recursive` and `determinant_impl` do now | 4.000000000000 | 0.013888888889 | 0.117851130198 |
| elimination as `gaussian_determinant` is written | **0.000000000000** | 0.000000000000 | **NaN** |
| elimination **with partial pivoting** | 4.000000000000 | 0.013888888889 | 0.117851130198 |

The exact regular-tetrahedron volume is √2⁄12 = 0.117851130198. The 4×4 case (a right triangle)
behaves identically: −4, 0, −4. The Gram matrix `lazy_hodge_star` actually feeds it agrees across all
three methods, which is why the missing pivoting has never shown up.

**The answer.** With partial pivoting the values are identical to Laplace on the shapes topology
uses — no numerical change at all. Without it, every volume collapses to zero. So the risk is not
"replacing Laplace perturbs rounding"; it is that `gaussian_determinant` is not a general determinant
and must not be treated as one. The shared implementation **must pivot**, and that requirement is
load-bearing rather than a quality nicety.

The reproduction script is `prototype/cm_det.py`.

## Blast radius

### The sparse crate

`deep_causality_sparse` is 3,107 lines of `src` and 1,916 of tests, published at 0.2.2 with 10,316
downloads (9,599 recent). Its public surface is six items: `CsrMatrix`, `SparseMatrixError`,
`CsrMatrixWitness`, `CsrFromTensorError`, and four `cg_solve*` functions.

The literal string `deep_causality_sparse` appears in **203 files, 435 lines** (excluding
`thirdparty/`, `target/`, `node_modules/` and stale agent worktrees):

| area | files |
|---|---|
| the crate itself | 30 |
| consuming crates | 95 |
| openspec archived changes | 34 |
| examples | 18 |
| build / CI / root | 11 |
| openspec notes and specs | 10 |
| docs and website | 5 |

Mentions are not edits. Of the 435 lines, **102 are `use deep_causality_sparse…` imports outside
the crate** — the code that must actually change:

| crate | files mentioning | import sites |
|---|---|---|
| `deep_causality_topology` | 73 | 61 (32 `src`, 29 `tests`) |
| `deep_causality_physics` | 5 | 2 |
| examples | 18 | 10 |
| `deep_causality_algebra` | 5 | 0 — doc comments naming laws `CsrMatrix` does not satisfy |
| discovery, cfd, and seven others | 1–2 each | 0 — prose and changelogs |

`deep_causality_topology` mentions `CsrMatrix` 282 times.

Bazel carries 35 label references across 8 `BUILD.bazel` files — 15 in
`deep_causality_topology/tests/`, 9 in `examples/mathematics_examples/`, 7 in the sparse crate's own
two files, and one each in topology, physics and cfd. One of them —
`deep_causality_cfd/BUILD.bazel:30` — declares a dependency that `deep_causality_cfd/Cargo.toml`
does not, so the two build systems currently disagree.

One published spec names the crate normatively: `openspec/specs/neumann-poisson/spec.md:34`.

### The tensor crate

`deep_causality_tensor` is published at 0.5.1 with 13,280 downloads. Eight in-workspace crates and
seven example crates depend on it. Relocating `svd`/`qr`/`eigen`/`inverse` off `CausalTensor` would
break every one of them, so the proposal keeps the methods and moves only their bodies.

### The `tensor-iso` feature is not load-bearing

`deep_causality_sparse` depends optionally on `deep_causality_tensor` through `tensor-iso`, gating
the `CausalTensor ↔ CsrMatrix` conversion in `extensions/ext_iso.rs`. **No library crate enables it.**
The only enablements are `examples/mathematics_examples/Cargo.toml:23`, for the
`tensor_sparse_memory_budget` example, and the sparse crate's own Bazel targets at `BUILD.bazel:10`
and `tests/BUILD.bazel:58` for its own tests.

So the conversion moves into `deep_causality_tensor` and the feature is deleted rather than
relocated. The gate exists so sparse users do not pay for a dependency on tensor; once tensor depends
on linear outright, that dependency is already paid and there is nothing left to gate.

### The name

`deep_causality_linear` returns 404 from the crates.io API — the name is free.

## Migration

crates.io cannot rename a crate. The path is a new crate plus a retirement, and per the user's
constraint the retired crate stays available so that already-published dependents keep resolving.

1. Publish `deep_causality_linear` 0.1.0 with the consolidated content.
2. Publish one final `deep_causality_sparse` whose `src/lib.rs` re-exports `deep_causality_linear`
   and whose README carries a retirement notice naming the successor.
3. Keep it in the workspace and on crates.io for the deprecation window — a few months. Nothing is
   yanked at any point.

The re-export matters beyond convenience. If the retired crate froze its own implementation instead,
`deep_causality_sparse::CsrMatrix` and `deep_causality_linear::CsrMatrix` would be distinct types,
and any crate depending on both — which is exactly what a partially-migrated dependent looks like —
would fail to typecheck. Re-exporting keeps one type.

Archived openspec changes under `openspec/changes/archive/` are historical records of what was
proposed at the time and are not rewritten. That leaves 34 of the 203 files untouched by design.

## Resolved

- **Do topology's small determinants change numerically?** No, provided the shared determinant does
  partial pivoting — see the section above, where pivoted elimination reproduces Laplace exactly on
  the Cayley-Menger matrices topology uses. Without pivoting every volume collapses to zero.
- **How much of the 1,088 tensor lines moves?** All of it. Nothing stays in `deep_causality_tensor`
  but the delegating method shells, and none is deferred to a follow-up change.
- **Does the retirement window end in a yank?** No. `deep_causality_sparse` is never yanked at any
  point.
- **Does a dense matrix type have real call sites?** Yes — 46 rank-2 constructions, and physics,
  quantum and topology call 56 two-dimensional operations and zero N-d ones. See the census above.
- **Do physics' five small-matrix helpers consolidate?** One pair does, inside
  `deep_causality_physics`, and it needs no crate boundary. Out of scope here.

## Open questions

1. **Are the 60 rank-1 sites worth a vector type?** The census answered the matrix question and
   raised this one. A dense matrix type serves none of them, and `Vec<T>` may simply be right.

## Related

- `openspec/notes/quantum/qcl-gaps.md` — G-01 (no 𝔽₂ linear algebra), G-02 (homology rank by f64 SVD)
- `openspec/specs/neumann-poisson/spec.md` — names `deep_causality_sparse` normatively
- `AGENTS.md` §Project Dependencies — the tier graph this change edits
