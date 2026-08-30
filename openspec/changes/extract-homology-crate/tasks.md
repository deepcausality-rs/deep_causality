## 1. Repair the basis orientation in `deep_causality_linear`

Independently useful and independently releasable. Do this first: it is a live defect, and the
extraction would otherwise carry it into a new crate.

- [ ] 1.1 Write a failing test: call `kernel_basis_gf2` on a matrix with a known kernel, read each basis vector, and assert `A · v = 0`. It fails today because the vectors are columns and `from_row` reads rows
- [ ] 1.2 Add `PackedGf2Vector::from_column`, reading a column across the word stride rather than copying a contiguous word slice
- [ ] 1.3 Add `Gf2Chain::from_column` delegating to it
- [ ] 1.4 Make 1.1 pass, and add the mirror test for `image_basis_gf2`
- [ ] 1.5 Correct the four docstrings that say rows: `packed_gf2_vector/mod.rs:107-108`, `gf2_chain/mod.rs:73`, `packed_gf2_vector_tests.rs:177`, `gf2_chain_tests.rs:100`
- [ ] 1.6 Move `widen_to_dense_i64` from `topology/src/types/homology_field/mod.rs` into `linear/src/extensions/conversions.rs` as `csr_i8_to_dense_i64`, exported from `lib.rs`
- [ ] 1.7 Bump `deep_causality_linear` to 0.1.2; `cargo test -p deep_causality_linear`, clippy, fmt

## 2. Create the crate skeleton

- [ ] 2.1 `deep_causality_homology/Cargo.toml` at 0.1.0 with description, license, repository, `[lints] workspace = true`, and path deps on `deep_causality_linear`, `deep_causality_algebra`, `deep_causality_num`
- [ ] 2.2 Add the crate to the workspace `members` list
- [ ] 2.3 `deep_causality_homology/BUILD.bazel` and `tests/BUILD.bazel` with a `rust_test_suite` per test directory
- [ ] 2.4 Register in `.github/workflows/formalization.yml` — omission produces a false `MISSING Rust witness` on every new theorem id
- [ ] 2.5 Register in `.github/workflows/rust_deps.yml` — omission is silent, cargo-machete simply never inspects the crate
- [ ] 2.6 Add the crate index entry and tier block to `AGENTS.md`
- [ ] 2.7 `cargo build -p deep_causality_homology` and `bazel build //deep_causality_homology/...` on an empty lib

## 3. Move the chain-complex layer

- [ ] 3.1 Move `HomologyField` and `rank_of` into `homology/src/types/homology_field/`, calling linear's `csr_i8_to_dense_i64` from step 1.6
- [ ] 3.2 Move the `ChainComplex` trait into `homology/src/traits/chain_complex.rs`, keeping only `num_cells`, `max_dim`, `boundary_matrix`, `coboundary_matrix`, and the provided `betti_number_over` and `betti_number`
- [ ] 3.3 Keep `boundary_matrix` and `coboundary_matrix` returning `Cow<'_, CsrMatrix<i8>>`; add no coefficient parameter
- [ ] 3.4 Move `Gf2Chain<W>` into `homology/src/types/gf2_chain/`
- [ ] 3.5 Introduce `HomologyError`, or re-use the linear error, so the crate does not depend on `TopologyError`
- [ ] 3.6 Move the corresponding tests, and add a probe asserting the crate's dependency set excludes `deep_causality_topology`

## 4. Add the topology seam

- [ ] 4.1 Add `deep_causality_homology` as a path dependency of `deep_causality_topology`
- [ ] 4.2 Define `pub trait CellComplex: ChainComplex` in `topology/src/traits/` with `CellType`, `CellIter`, `Metric`, `cells(k)` and `uniform_lattice_layout()`
- [ ] 4.3 Re-export `ChainComplex`, `HomologyField` and `Gf2Chain` from `topology/src/lib.rs`
- [ ] 4.4 Split each of the three implementors — `SimplicialComplex`, `CellComplex<C>`, `LatticeComplex` — into a `ChainComplex` impl and a `CellComplex` impl
- [ ] 4.5 Build `deep_causality_cfd` and `deep_causality_physics` with **no** `Cargo.toml` edit and **no** `use` edit. A required edit means the re-export in 4.3 is wrong
- [ ] 4.6 Bump `deep_causality_topology` to 0.7.4

## 5. Settle `Gf2Chain`'s complex identity

- [ ] 5.1 Decide the carrier: `Arc<Complex>` as `Chain<T>` uses, or a cheaper identity such as a shape tuple or generation counter. `Chain<T>`'s `Arc::ptr_eq` needs a concrete complex type, which generic `Gf2Chain<W>` does not have
- [ ] 5.2 Implement it and make `add`, `intersect` and `inner` reject operands from different complexes
- [ ] 5.3 Test: two chains of equal degree and equal length from two different complexes are refused rather than answered

## 6. State and test the differential law

- [ ] 6.1 State `∂ₖ ∘ ∂ₖ₊₁ = 0` in the `ChainComplex` docstring and in `Cell`'s, since `CellComplex<C>` derives every operator from `Cell::boundary()`
- [ ] 6.2 Assert it in the conformance harness for every implementor at every grade, widening coefficients past `i8` before multiplying so the assertion does not run on wrapping arithmetic in release
- [ ] 6.3 Add `CellComplex` to the conformance harness with a concrete `Cell` fixture; it is absent today
- [ ] 6.4 Verify the harness discriminates: a deliberately malformed complex must fail it

## 7. Fix the degenerate-grade shape

- [ ] 7.1 Change all three implementors so `∂₀` has shape `(0, num_cells(0))` and `∂_{max+1}` has shape `(num_cells(max), 0)`, in place of `CsrMatrix::new()`
- [ ] 7.2 Extend `assert_shape_invariant` from `1..=max_d` to cover both ends
- [ ] 7.3 Assert every Betti number is unchanged at every grade of every shipped complex over both fields
- [ ] 7.4 Repair `lattice_complex_test.rs:183-193`, which asserts `cols(∂₂) == rows(∂₁)` — comparing `n₂` with `n₀` — and passes on a torus by coincidence. Composability of `∂₁ ∘ ∂₂` is `cols(∂₁) == rows(∂₂)`

## 8. Verify and record

- [ ] 8.1 `bazel test //...` — every test passes, and the count is at least the pre-change count
- [ ] 8.2 `cargo clippy --workspace --all-targets -- -D warnings`, fixing by rewriting rather than by `#[allow]`
- [ ] 8.3 `cargo fmt --all -- --check`
- [ ] 8.4 Generate the SBOM pair for the new crate
- [ ] 8.5 Record in `openspec/notes/quantum/qcl-gaps.md` that G-04 and G-08 now have a home, and that the `deep_causality_quantum` dependency edge goes to homology rather than topology
- [ ] 8.6 Prepare the commit message; do not commit
