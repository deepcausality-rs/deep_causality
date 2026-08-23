## 1. Stand up the crate and retire the old name

The workspace builds against the new name and the old name still works. Nothing new is implemented
here; this phase is a move plus a facade.

- [ ] 1.1 Create `deep_causality_linear` with `[lints] workspace = true`, `no_std` + `alloc` feature parity with `deep_causality_sparse`, and dependencies on `deep_causality_num`, `deep_causality_algebra`, `deep_causality_haft` only
- [ ] 1.2 Move `CsrMatrix`, its arithmetic, ops, getters, identity, display and algebra impls from `deep_causality_sparse/src/types/sparse_matrix/`
- [ ] 1.3 Move `solver/cg.rs` (`cg_solve`, `cg_solve_preconditioned`, `cg_solve_preconditioned_from`, `CgFailure`) and `errors/sparse_matrix_error.rs`
- [ ] 1.4 Move `extensions/ext_hkt.rs` (`CsrMatrixWitness`) and its tests
- [ ] 1.5 Move `extensions/ext_iso.rs` and `CsrFromTensorError` into `deep_causality_tensor` unconditionally; delete the `tensor-iso` feature and its `#[cfg]` gates, the `"tensor-iso"` entries in `deep_causality_sparse/BUILD.bazel:10` and `tests/BUILD.bazel:58`, and the `features = ["tensor-iso"]` on `examples/mathematics_examples/Cargo.toml:23`
- [ ] 1.6 Move the 1,916 lines of `deep_causality_sparse/tests/` and the `csr_matrix_benchmarks` bench
- [ ] 1.7 Reduce `deep_causality_sparse/src/lib.rs` to re-exports of `deep_causality_linear`; confirm the public surface matches the last independent release item for item
- [ ] 1.8 Write the retirement notice at the top of `deep_causality_sparse/README.md`, naming `deep_causality_linear` and stating that the crate receives no further development
- [ ] 1.9 Switch the 102 import sites: `deep_causality_topology` 61 (32 `src`, 29 `tests`), examples 10, `deep_causality_physics` 2 (`kernels/mhd/ideal.rs:11`, `kernels/mhd/grmhd.rs:11`), and the 29 inside the crate being moved
- [ ] 1.10 Add `deep_causality_linear/BUILD.bazel` and retarget the 35 label references across the 8 `BUILD.bazel` files that name the old crate
- [ ] 1.11 Resolve the `deep_causality_cfd` discrepancy: `BUILD.bazel:30` declares a dependency `Cargo.toml` does not — decide which is correct and make both agree
- [ ] 1.12 Update `AGENTS.md` §Project Structure and §Project Dependencies, `README.md:268`, `website/web/src/pages/overview/index.astro` (2 sites), `website/docs/…/getting-started/install.md`, `website/docs/…/concepts/uniform-math.md`
- [ ] 1.13 Leave the 34 files under `openspec/changes/archive/` unchanged; confirm by diff that none were touched
- [ ] 1.14 Register the crate in `build/scripts/sbom.sh`, and add the two crates missing from it (`deep_causality_num_rational`, `deep_causality_quantum`) — it lists 26 against a workspace of 29
- [ ] 1.15 `cargo test --workspace` and `bazel test //...` green; clippy clean with no new `#[allow]`

## 2. Add the representations

- [ ] 2.1 Define the read trait: row count, column count, element access by value (a bit-packed representation has no element to lend a reference to)
- [ ] 2.2 Define the row-operation trait: `swap_rows`, `scale_row`, `axpy_rows`, and an overridable `pivot_in_column` defaulting to the first non-zero
- [ ] 2.3 Define the build trait: `zeros`, `set`, and a default `identity`, split from the row-operation trait so a read-only borrowed matrix can still be eliminated by copy
- [ ] 2.4 Add the dense row-major matrix; implement all three traits, with magnitude pivoting for float scalars
- [ ] 2.5 Add the bit-packed 𝔽₂ matrix generic over a `NaturalNumber` word type; implement all three traits with whole-word row updates
- [ ] 2.6 Implement the read trait for `CsrMatrix`; do not implement the row-operation trait for it, and document why in the module header
- [ ] 2.7 Add conversions: sparse ↔ dense (total), dense → 𝔽₂ and sparse → 𝔽₂ (fallible, naming the offending position), 𝔽₂ → dense (total)
- [ ] 2.8 Test the word-boundary cases: a column count that is not a multiple of the word width, and the same matrix at two different word widths agreeing on rank and pivot columns
- [ ] 2.9 Test that packing accepts `{-1, 0, 1}` reduced mod 2 and rejects anything else

## 3. Add elimination

- [ ] 3.1 Implement `rref` returning rank and pivot columns, generic over the row-operation trait, naming no concrete representation or scalar
- [ ] 3.2 Implement `rank`, `kernel_basis`, `image_basis`, `determinant` and `solve` over the same trait
- [ ] 3.3 Pivot by column search in every elimination path; add the Cayley-Menger regression first — a 5×5 CM matrix has `m[0][0] = 0`, and an unpivoted elimination returns zero for every simplex volume (`prototype/cm_det.py`)
- [ ] 3.4 Keep closed-form determinants for order ≤ 3 and dispatch to pivoted elimination at order ≥ 4; assert bit-identical results against the expressions they replace
- [ ] 3.5 Verify the 𝔽₂ kernel basis: `M · v = 0` over 𝔽₂ for every returned `v`, and the basis has exactly `cols − rank` elements
- [ ] 3.6 Verify the 𝔽₂ image basis: exactly `rank` elements, and every column of `M` is an 𝔽₂ sum of them
- [ ] 3.7 Port the prototype's four-way benchmark into `benches/`; record packed vs byte-scalar at n ∈ {128, 256, 512, 1024, 2048} and the seam cost against a hand-written non-generic loop
- [ ] 3.8 Confirm the seam requirement holds: generic packed elimination within 10% of hand-written (the prototype measured 0.92–0.95×)
- [ ] 3.9 Confirm the packing requirement holds: ≥ 2× against the byte scalar at n=1024, and no smaller a ratio at n=2048

## 4. Relocate the decompositions

- [ ] 4.1 Record the tensor benchmark baseline on the benchmark machine before touching anything
- [ ] 4.2 Move the bodies of `svd` (117), `svd_decomp` (170), `svd_truncated` (375), `qr` (145), `eigen` (158) and `inverse` (123) into `deep_causality_linear`
- [ ] 4.3 Add `deep_causality_linear` as a dependency of `deep_causality_tensor`; implement the read trait for `CausalTensor` there
- [ ] 4.4 Reduce `CausalTensor`'s inherent methods and the `Tensor` trait members at `traits/tensor.rs:435,439` to delegations; keep every signature, return shape and error variant
- [ ] 4.5 Expose the decompositions on the dense matrix directly, with no rank-2 tensor constructed
- [ ] 4.6 Re-run the tensor benchmarks and diff against 4.1; record both figures with the machine
- [ ] 4.7 Rebuild the 8 in-workspace and 7 example dependents with no source edit and confirm results are unchanged
- [ ] 4.8 Spot-check `deep_causality_physics` specifically — 18 `matmul` and 2 `inverse()` call sites, the largest delegation surface outside tensor's own tests
- [ ] 4.9 Leave physics' five hand-rolled fixed-size helpers alone (`invert_4x4`, `invert_3x3`, `inverse_spatial_metric`, `symmetric_3x3_eigenvalues`, the inline 3×3 determinant); they are correct and out of scope

## 5. Retire the duplication in topology

- [ ] 5.1 Replace `regge_geometry/curvature.rs:275` `det_recursive` and its `submatrix` helper with the shared determinant
- [ ] 5.2 Replace `manifold/geometry/mod.rs:145` `determinant_impl` with the shared determinant
- [ ] 5.3 Replace `simplicial_complex/lazy_hodge_star.rs:97` `gaussian_determinant` with the shared determinant
- [ ] 5.4 Confirm the Cayley-Menger volumes are unchanged: `regge_geometry` (5×5, tetrahedron) and `manifold/geometry` (`k + 2`) both feed matrices with a zero `(0,0)` entry
- [ ] 5.5 Diff the topology suite before and after 5.1–5.4; investigate every changed value rather than re-baselining
- [ ] 5.6 Replace `chain_complex_impl.rs:94` `rank_of_csr` and `cell_complex/mod.rs:172` `rank_of_matrix` with one implementation
- [ ] 5.7 Route `betti_number` through exact 𝔽₂ rank; make the choice of field explicit at the call site rather than a global default
- [ ] 5.8 Confirm every complex currently under test reports the Betti numbers it reported before
- [ ] 5.9 Confirm no floating-point tolerance remains on any rank path used for homology
- [ ] 5.10 Mark G-01 and G-02 closed in `openspec/notes/quantum/qcl-gaps.md`, correcting G-01's owner field from `deep_causality_topology` to `deep_causality_linear` and recording why
- [ ] 5.11 Add a Lean theorem and Rust witness for mod-2 rank–nullity if the formalization layer covers it; otherwise record the omission in the crate's `LEAN_*.md` and add the crate to the `formalization.yml` allowlist only when a witness exists

## 6. Publish

- [ ] 6.1 Publish `deep_causality_linear` 0.1.0 first — release-plz strips path dependencies when verifying publish tarballs, so each dependent resolves the published API of the crate below it
- [ ] 6.2 Publish the final `deep_causality_sparse` carrying the re-exports and the retirement notice
- [ ] 6.3 Publish `deep_causality_tensor`, `deep_causality_topology`, `deep_causality_physics` in dependency order
- [ ] 6.4 Verify a previously published dependent still resolves and compiles from crates.io
- [ ] 6.5 Confirm nothing was yanked at any point
