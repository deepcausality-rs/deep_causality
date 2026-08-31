<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Reading the archive: where the code went

**What this is.** A path translation table for `openspec/changes/archive/`. An archived change names
the files it touched, and some of those files have since moved to another crate. This file records
the moves so a reader can follow an old path to its current home.

**Why the archive is not rewritten.** An archived change records what was decided and done at a
point in time. `deep_causality_topology/src/traits/chain_complex.rs` was the right path when the
change that names it was written, and editing that line to point somewhere else would make the
record describe something that never happened. The paths stay. This file explains them.

One hundred and twenty-four archived documents name a path that has since moved. None of them is
wrong.

---

## 1. Crate-level moves

Ordered oldest first. "Files" counts source files git detected as renames across a crate boundary.

| Date | Commit | Move | Files |
|---|---|---|---|
| 2025-09-02 | `76d8fdcfe` | `dcl_data_structures` renamed to `deep_causality_data_structures` | 55 |
| 2025-09-14 | `def2d8acc` | `dcl_data_structures` retired to `yanked/` | — |
| 2025-12-05 | `5013f3cc6` | Effect ethos split out of `deep_causality` into `deep_causality_ethos` | 60 |
| 2026-06-04 | `db279aba8` | Calculus split out of `deep_causality_num` into `deep_causality_calculus` | 3 |
| 2026-06-12 | `327c055b2` | Parallelism marker split out of `deep_causality_topology` into `deep_causality_par` | 1 |
| 2026-07-08 | `5a4dce4b8` | **The num split.** `deep_causality_num` divided four ways | 188 |
| 2026-07-19 | `517579d98` | Superseded CFD binaries moved to `deep_causality_cfd/reverted/` | — |
| 2026-08-24 | `f01304c53` | `deep_causality_effects` retired to `yanked/` | — |
| 2026-08-24 | `d071e59d9` | `deep_causality_linear` declared | — |
| 2026-08-25 | `a02944ee5` | **The linear migration.** Workspace repointed onto `deep_causality_linear` | — |
| 2026-08-25 | `21cf19e36` | `deep_causality_macros` retired to `yanked/` | — |
| 2026-08-30 | `5d591e900` | **The homology extraction.** Chain complexes split out of topology and linear | — |
| 2026-08-30 | this change | **The unified-math consolidation.** Seventeen crates moved under `deep_causality_unified_math/` | 1856 |
| 2026-08-30 | this change | `deep_causality_sparse` retired to `yanked/` | 9 |

Data structures also gave 36 files to `deep_causality_tensor`, physics gave 8 to
`deep_causality_cfd` and 4 to `deep_causality_quantum`, and several crates gave example binaries to
`examples/`. Those are ordinary code movements rather than crate identity changes, and a reader
following a path through them will find the name unchanged in its new crate.

## 2. The four moves worth knowing about

### 2.1 The num split, 2026-07-08

`deep_causality_num` held the numeric traits, the algebra tower and the extended number types. It
now holds only the numeric traits. Three crates came out of it.

| Old path | Now in |
|---|---|
| `deep_causality_num/src/algebra/…` | `deep_causality_unified_math/deep_causality_algebra/src/algebra/…` |
| `deep_causality_num/src/complex/…` | `deep_causality_unified_math/deep_causality_num_complex/src/complex/…` |
| `deep_causality_num/src/dual/…` | `deep_causality_unified_math/deep_causality_num_dual/src/dual/…` |
| `deep_causality_num/src/rational/…` | `deep_causality_unified_math/deep_causality_num_rational/src/…` |

Package names in `use` statements changed with the paths. An archived change that says
`use deep_causality_num::Field` predates the split; the trait is `deep_causality_algebra::Field`.

Integer algebra impls were part of the experiment and were deliberately not kept. An archived
change that discusses an integer tower describes a road not taken.

### 2.2 The linear migration, 2026-08-24 to 2026-08-25

`deep_causality_sparse` held the CSR matrix and the conjugate gradient solver.
`deep_causality_linear` now holds those plus the dense, bit-packed 𝔽₂ and vector representations,
the eliminations, the decompositions and the exact integer path.

| Old path | Now in |
|---|---|
| `deep_causality_sparse/src/types/sparse_matrix/` | `deep_causality_unified_math/deep_causality_linear/src/types/csr_matrix/` |
| `deep_causality_sparse/src/solver/cg.rs` | `deep_causality_unified_math/deep_causality_linear/src/algorithms/cg.rs` |
| `deep_causality_sparse/src/errors/sparse_matrix_error.rs` | `deep_causality_unified_math/deep_causality_linear/src/errors/` |
| `deep_causality_sparse/src/extensions/ext_hkt.rs` | `deep_causality_unified_math/deep_causality_linear/src/extensions/ext_hkt.rs` |
| `deep_causality_sparse/src/extensions/ext_iso.rs` | `deep_causality_unified_math/deep_causality_tensor/src/extensions/ext_iso.rs` |

Two names changed shape rather than address. `SparseMatrixError` became `LinearError`, one error
type across all representations; `CgFailure` became an enum. The retired crate's `src/lib.rs` carries
the full mapping table, and it is still readable at `yanked/deep_causality_sparse/src/lib.rs`.

### 2.3 The homology extraction, 2026-08-30

`ChainComplex` used to carry eleven items, six about homology and five about geometry. The homology
half moved. The geometry half stayed on `CellularComplex`.

| Old path | Now in |
|---|---|
| `deep_causality_topology/src/traits/chain_complex.rs` | `deep_causality_unified_math/deep_causality_homology/src/traits/chain_complex.rs` |
| `deep_causality_topology/src/types/homology_field/…` | `deep_causality_unified_math/deep_causality_homology/src/types/homology_field/…` |
| `deep_causality_topology/src/types/gf2_chain/…` | `deep_causality_unified_math/deep_causality_homology/src/types/gf2_chain/…` |
| `deep_causality_linear/src/extensions/conversions.rs` (𝔽₂ arms) | shared with `deep_causality_homology` |

`deep_causality_unified_math/deep_causality_topology/src/traits/chain_complex.rs` still exists as a re-export shim, so
`use deep_causality_topology::ChainComplex` in an archived change still compiles. `Chain<T>`,
`SimplicialComplex<T>` and `cup_product` did not move; they need cells, and a chain complex has
none.

### 2.4 `deep_causality_sparse` retired, 2026-08-30

Moved to `yanked/deep_causality_sparse`, marked `publish = false`, and removed from the workspace
members. Its contents had already moved in §2.2; what moved now is a 69-line re-export shim.

**It is not yanked from crates.io.** Version 0.2.5 stays published and keeps working for any
existing dependent. It is the last release, and no further one is planned. Callers should repoint to
`deep_causality_linear`.

Two example crates carried a dependency on it that no source file used. Both were removed in the
same change, along with its entry in the `cargo machete` argument list in
`.github/workflows/rust_deps.yml`.

## 3. Note directories

The design notes moved too, and archived changes link to them.

| Old path | Now in |
|---|---|
| `openspec/notes/linear/` | `openspec/notes/archive/linear/` |
| `openspec/notes/homology/` | `openspec/notes/archive/homology/` |
| `openspec/notes/unified_math/HKT-LAW-FINDINGS.md` | `openspec/notes/unified_math/HKT-LAW-FINDINGS.md` |
| `specs/` | `openspec/` |

## 4. The unified-math consolidation, 2026-08-30

The seventeen mathematics crates moved from the repository root into `deep_causality_unified_math/`.
Package names did not change, so no `use` statement moved and nothing on crates.io was affected.
Only directories moved.

Every archived path that begins with one of these seventeen names gains the folder prefix:

```
deep_causality_{algebra, ast, calculus, fft, haft, homology, linear, metric, multivector,
                num, num_complex, num_dual, num_rational, rand, tensor, topology, uncertain}
```

So an archived change naming `deep_causality_topology/src/types/chain/mod.rs` now means
`deep_causality_unified_math/deep_causality_topology/src/types/chain/mod.rs`. The rule is uniform:
prepend `deep_causality_unified_math/` and change nothing else.

The twelve crates that stayed at the root are `deep_causality`, `_algorithms`, `_cfd`, `_core`,
`_data_structures`, `_discovery`, `_ethos`, `_file`, `_par`, `_physics`, `_quantum` and
`ultragraph`. A path beginning with one of those is still correct as written.

The assessment that preceded the move is
`openspec/notes/unified_math/deep_causality_unified_math.md`.

## 5. Keeping this file true

Add a row when a change moves code across a crate boundary, retires a crate, or relocates a note
directory. Do not add a row for a file that moved inside its own crate; the crate name is what makes
an archived path hard to follow, and a reader who has the right crate can find the file.

The archive itself stays as written.
