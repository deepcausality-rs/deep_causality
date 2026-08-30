## Why

Homology is split across two crates. `deep_causality_topology` owns the concepts — `ChainComplex`,
`HomologyField`, `Gf2Chain` — and delegates every computation to `deep_causality_linear`, which in
turn carries conversion helpers whose docstrings name topology's boundary operators as their reason
to exist. Neither crate owns the seam, so half of it sits in each.

The cost lands now. `deep_causality_quantum` depends on neither crate today, and QCL gaps G-07 and
G-09 force it to take that edge. Six of Haruna's nine requirements (R1–R6) are pure homology, and a
quantum error-correcting code is a chain complex that is not a space: `H_X` and `H_Z` are parity-check
matrices with no cells, no metric and no Hodge star. Without this change, `deep_causality_quantum`
must depend on 27,317 lines of geometry to reach 419 lines of chain-complex machinery.

## What Changes

- **New crate `deep_causality_homology`** holding the chain-complex layer: the `ChainComplex` trait
  over boundary matrices only, `HomologyField`, and `Gf2Chain<W>`.
- **`ChainComplex` splits in two.** Its six homology items (`num_cells`, `max_dim`,
  `boundary_matrix`, `coboundary_matrix`, and the provided `betti_number_over` / `betti_number`)
  move to the new crate. Its five geometry items (`CellType`, `CellIter`, `Metric`, `cells`,
  `uniform_lattice_layout`) stay in topology on a new `CellularComplex: ChainComplex` supertrait.
- **`deep_causality_topology` re-exports** `ChainComplex`, `HomologyField` and `Gf2Chain`, so no
  downstream crate needs an import change. Not a breaking change.
- **`widen_to_dense_i64`** moves from a private helper in topology to
  `deep_causality_linear::extensions::conversions` beside the other five conversions, as
  `csr_i8_to_dense_i64`.
- **`PackedGf2Vector::from_column` is added** and four docstrings corrected. `kernel_basis_gf2` and
  `image_basis_gf2` return **column** bases; four committed docstrings say rows, and `from_row` is a
  contiguous word-slice copy that cannot read a column. This is a live defect.
- **`Gf2Chain` keeps `(degree, len)` as the identity of its chain group**, and gains one guard for
  it. `C_k = 𝔽₂^{n_k}` is fixed by the cell count, and every operation the type has belongs to that
  group rather than to a complex. Today the degree is checked in the chain and the length in the
  packed vector beneath it, so one condition surfaces as two error types. Carrying an `Arc`, a
  generated id or a phantom brand is rejected, with reasons, in design Decision 5.
- **`∂∘∂ = 0` becomes a stated law** on the trait and an assertion in the conformance harness.
- `Chain<T>` and `cup_product` stay in topology. Both are geometry-coupled: `Chain<T>` holds an
  `Arc<SimplicialComplex<T>>` and compares with `Arc::ptr_eq`; the cup product needs
  `SplittableCell::split` for the Alexander–Whitney diagonal, and a chain complex has no
  multiplication.
- **`boundary_matrix` keeps `Cow<'_, CsrMatrix<i8>>`.** The entries are incidence numbers, and for a
  cell complex they are `{-1, 0, 1}` by construction. That is an invariant of the boundary operator
  rather than a storage convenience, so `i8` is the honest type and the trait needs no coefficient
  parameter. `HomologyField` chooses the field at the call site, where the choice actually lives.
  Parameterising `ChainComplex` over `R` is rejected, not deferred.

## Capabilities

### New Capabilities
- `homology-chain-complex`: the `ChainComplex` trait stated over boundary matrices alone, the
  `∂∘∂ = 0` law, `HomologyField` and its two coefficient arms, and Betti numbers over a chosen field.
- `homology-gf2-chain`: the bit-packed mod-2 chain bound to the chain group `(degree, len)`
  identifies, with support enumeration, the mod-2 pairing and intersection.
- `topology-cell-complex-seam`: the geometric supertrait in `deep_causality_topology` and the
  re-exports that keep every existing consumer compiling unchanged.

### Modified Capabilities
<!-- None. No spec in openspec/specs/ covers chain complexes or homology today; the linear crate's
     specs remain under the unarchived add-linear-algebra-crate change. -->

## Impact

**New crate.** `deep_causality_homology` 0.1.0. Depends on `deep_causality_linear`,
`deep_causality_algebra`, `deep_causality_num`. Needs `BUILD.bazel`, `tests/BUILD.bazel`, an SBOM
pair, `[lints] workspace = true`, an AGENTS.md index and tier entry, and entries in two
hand-maintained CI lists: `.github/workflows/formalization.yml` (omission produces a false
`MISSING Rust witness`) and `.github/workflows/rust_deps.yml` (omission is silent).

**Modified.** `deep_causality_topology` 0.7.3 → 0.7.4, additive given the re-exports.
`deep_causality_linear` 0.1.1 → 0.1.2 for `from_column` and `csr_i8_to_dense_i64`.

**Unaffected.** `deep_causality_cfd` and `deep_causality_physics` name `ChainComplex` in 12 files and
call only `num_cells` (18 sites) and `uniform_lattice_layout` (1). No crate outside topology calls
`boundary_matrix`, `coboundary_matrix`, `betti_number` or `betti_number_over`. All four dependents
pin `version = "0.7"`, so a patch is picked up with no file touched. **Zero dependent `Cargo.toml`
edits.**

Two `use` edits are needed, both in `deep_causality_cfd`, and no re-export avoids either.
A method belongs to one trait, and calling it needs that trait in scope; re-exporting a name does
not change which trait owns a method.

- `src/solvers/dec/spectral_diffusion.rs` calls `uniform_lattice_layout()`.
- `tests/solvers/dec/cut_cell_wiring_tests.rs` calls `cells()` at four sites.

Both were importing `ChainComplex` while using geometry. They now import `CellularComplex`, which
has `ChainComplex` as a supertrait, so nothing else in either file changed. Every other consumer —
all 18 `num_cells` sites, both `deep_causality_physics` files, and the rest of the CFD tree —
resolves through the re-export untouched.

**Unblocks.** QCL gaps G-04 (homology representatives) and G-08 (Poincaré-dual representative) gain a
home that `deep_causality_quantum` can depend on without inheriting geometry. Neither is in scope
here.

**Reference.** The reviewed inventory and measurements are in
`openspec/notes/homology/extraction-plan.md`.
