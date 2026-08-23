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
group — 15 call sites in the physics Kalman filter alone
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

A prototype in `.claude/worktrees/wf_d43bff9e-2a0-2/proto/` runs one generic `rref`, written against
a row-operation trait, over four representations of the same 𝔽₂ matrix. M3 Max, 16 cores, 128 GB;
`--release`.

| n | packed `u64`, generic | packed, hand-written | `Dense<Gf2>` (1 byte/bit) | `Vec<Vec<Gf2>>` | packed vs byte | seam cost | memory |
|---|---|---|---|---|---|---|---|
| 128 | 103.25 µs | 110.50 µs | 170.58 µs | 283.25 µs | 1.7× | 0.93× | 2 vs 16 KiB |
| 256 | 410.50 µs | 447.79 µs | 702.50 µs | 1.173 ms | 1.7× | 0.92× | 8 vs 64 KiB |
| 512 | 1.803 ms | 1.896 ms | 3.341 ms | 5.261 ms | 1.9× | 0.95× | 32 vs 256 KiB |
| 1024 | 7.869 ms | 8.494 ms | 18.685 ms | 26.682 ms | 2.4× | 0.93× | 128 vs 1024 KiB |
| 2048 | 34.748 ms | 37.311 ms | 112.517 ms | 148.227 ms | 3.2× | 0.93× | 512 vs 4096 KiB |

Two results.

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
else — `proto/tensor_impl/` compiles the third case and gets E0117. The two legal placements force
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
| openspec archived changes | 36 |
| examples | 18 |
| build / CI / root | 11 |
| openspec notes and specs | 8 |
| docs and website | 5 |

Of those, 102 are `use deep_causality_sparse…` import lines outside the crate. 67 of the 95
consuming-crate files are `deep_causality_topology`, which mentions `CsrMatrix` 279 times.
`deep_causality_physics` has 2 import sites and 11 `CsrMatrix` mentions. `deep_causality_algebra`
mentions `CsrMatrix` 4 times, all in doc comments explaining which algebraic laws it does not
satisfy.

Bazel carries 30 label references across 8 `BUILD.bazel` files. One of them —
`deep_causality_cfd/BUILD.bazel:30` — declares a dependency that `deep_causality_cfd/Cargo.toml`
does not, so the two build systems currently disagree.

One published spec names the crate normatively: `openspec/specs/neumann-poisson/spec.md:34`.

### The tensor crate

`deep_causality_tensor` is published at 0.5.1 with 13,280 downloads. Eight in-workspace crates and
seven example crates depend on it. Relocating `svd`/`qr`/`eigen`/`inverse` off `CausalTensor` would
break every one of them, so the proposal keeps the methods and moves only their bodies.

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
proposed at the time and are not rewritten. That leaves 36 of the 203 files untouched by design.

## Open questions

1. **Does the retired crate keep its `tensor-iso` feature?** Today `deep_causality_sparse` depends
   optionally on `deep_causality_tensor`. Once tensor depends on linear, linear cannot depend on
   tensor even optionally, so the `CausalTensor ↔ CsrMatrix` conversion has to move into
   `deep_causality_tensor`. A re-export facade cannot re-export a feature whose code moved to a
   crate above it.
2. **Do topology's small determinants change numerically?** Replacing the two O(n!) Laplace
   expansions with elimination perturbs floating-point results. Regge geometry uses them on
   Cayley-Menger determinants of small simplices where Laplace is also the faster path.
3. **How much of the 1,088 tensor lines moves in the first change?** Delegation is behaviour-preserving
   but touches the most-used numerical code in the workspace.
4. **Does the retirement window end in a yank?** The stated intent is a few months of availability.
   Nothing in this repository's conventions requires a yank afterwards, and yanking would break the
   already-published dependents the window exists to protect.

## Related

- `openspec/notes/quantum/qcl-gaps.md` — G-01 (no 𝔽₂ linear algebra), G-02 (homology rank by f64 SVD)
- `openspec/specs/neumann-poisson/spec.md` — names `deep_causality_sparse` normatively
- `AGENTS.md` §Project Dependencies — the tier graph this change edits
