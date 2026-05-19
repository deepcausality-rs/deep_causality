## 1. Part A — Static-dispatch ChainComplex trait (pure refactor, no behavior change)

- [ ] 1.1 Rename `src/traits/cw_complex.rs` → `src/traits/chain_complex.rs`; rename trait `CWComplex` → `ChainComplex` and marker trait `Cell` is left as-is.
- [ ] 1.2 Replace `fn cells(&self, k: usize) -> Box<dyn Iterator<Item = Self::CellType> + '_>` with associated `type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a;` and `fn cells(&self, k: usize) -> Self::CellIter<'_>`.
- [ ] 1.3 Add `fn coboundary_matrix(&self, k: usize) -> CsrMatrix<i8>` to the `ChainComplex` trait contract.
- [ ] 1.4 Update `src/traits/mod.rs` to export `ChainComplex` and remove `CWComplex`; update root `lib.rs` re-exports.
- [ ] 1.5 Update `Lattice<D>`'s impl in `src/types/lattice/mod.rs` to the new trait surface: introduce a concrete `LatticeCellIter<'a, D>` (already exists as `LatticeCellIterator`) as `type CellIter<'a>`, drop the `Box::new`, add `coboundary_matrix(k)` returning `boundary_matrix(k+1).transpose()` (memoized lazily — `RefCell<HashMap<usize, CsrMatrix<i8>>>` if simple; otherwise recompute).
- [ ] 1.6 Implement `ChainComplex` for `SimplicialComplex<C>` in `src/types/simplicial_complex/topology/` (new file `chain_complex_impl.rs`). `coboundary_matrix(k)` returns a clone of `self.coboundary_operators[k]`. `cells(k)` returns the concrete simplicial-cell iterator.
- [ ] 1.7 Migrate every existing call site of `CWComplex` to `ChainComplex` (audit with `grep -RIn "CWComplex" src/ tests/`); confirm no `Box<dyn Iterator>` remains in topology trait surfaces.
- [ ] 1.8 Update `tests/traits/cw_complex_tests.rs` → `chain_complex_tests.rs`; register in `tests/traits/mod.rs` and `tests/BUILD.bazel`. Add a test that asserts `size_of` of the returned iterator from `cells(k)` is non-zero and not heap-allocated (use a static-dispatch sanity check: pass `K: ChainComplex` to a generic helper).
- [ ] 1.9 Run `cargo build -p deep_causality_topology` and `cargo test -p deep_causality_topology`; both must pass with zero behavior change.
- [ ] 1.10 Run `make format && make fix` and verify no clippy warnings introduced.

## 2. Part B — Genericize Manifold over ChainComplex

- [ ] 2.1 In `src/types/manifold/mod.rs`, change the struct from `Manifold<C, D>` with `complex: SimplicialComplex<C>` to `Manifold<K: ChainComplex, F>` with `complex: K`, `data: CausalTensor<F>`, `metric: Option<ReggeGeometry<K>>` (or `Option<Box<...>>` only if forced — prefer trait-generic metric; see 2.5), `cursor: usize`.
- [ ] 2.2 Add public type alias `pub type SimplicialManifold<C, F> = Manifold<SimplicialComplex<C>, F>;` in `src/types/manifold/mod.rs`; re-export from `lib.rs`.
- [ ] 2.3 Update every file under `src/types/manifold/{api,constructors,covariance,display,geometry,getters,utils,differential,topology}/` to take `K: ChainComplex` instead of the concrete `SimplicialComplex<C>`. Mechanical sed-style edits — no logic change.
- [ ] 2.4 In `src/types/manifold/differential/exterior_cpu.rs`, replace `&self.complex.coboundary_operators[k]` with `self.complex.coboundary_matrix(k)`. Repeat for `codifferential_cpu.rs`, `hodge_cpu.rs`, `laplacian_cpu.rs` — all reads of pre-cached boundary/coboundary go through trait methods.
- [ ] 2.5 Resolve `ReggeGeometry<K>` genericization: either (a) keep `ReggeGeometry` simplicial and introduce a `MetricKind<K>` enum at the `Manifold` level holding `Simplicial(ReggeGeometry<...>)` or `Cubical(CubicalMetric<D>)`, or (b) genericize `ReggeGeometry<K: ChainComplex>` directly. Pick (a) — it isolates the diff. Create `src/types/cubical_metric/mod.rs` carrying the unit-edge cubical metric (constant edge length `1.0`).
- [ ] 2.6 Update `src/extensions/hkt_manifold/mod.rs` to re-parameterize `ManifoldWitness` and the `Functor`/`CoMonad` impls over `K: ChainComplex` instead of `C: Satisfies<NoConstraint>`. Iteration order and cursor semantics MUST remain identical.
- [ ] 2.7 Update every existing import/use of `Manifold<C, D>` to `SimplicialManifold<C, D>` across `deep_causality_topology` and any downstream monorepo crate (`grep -RIn "Manifold<" .`).
- [ ] 2.8 Update test files under `tests/types/manifold/` to use `SimplicialManifold`; do not rename test files (the underlying type is still being exercised).
- [ ] 2.9 Add an assertion test that `grep -RIn "complex.coboundary_operators" src/types/manifold/` returns zero matches — encoded as a Bazel `genrule` test or a Rust `build.rs` guard if the repo already has that pattern; otherwise document the check in `tests/types/manifold/no_direct_field_access_tests.rs` as a `cargo test` integration that scans the source.
- [ ] 2.10 Run `cargo build -p deep_causality_topology && cargo test -p deep_causality_topology` — every pre-existing simplicial test must pass unchanged.
- [ ] 2.11 Run `make build` to confirm no downstream monorepo crate breaks (this is where the import migration in 2.7 is verified end-to-end).
- [ ] 2.12 Run `make format && make fix`.

## 3. Part C — Cubical surfacing, Neighborhood strategy, and example

- [ ] 3.1 Move `src/types/lattice/` → `src/types/cubical_complex/` (file moves via `git mv`); rename `lattice_cell.rs` → `cubical_cell.rs`, `dual_lattice.rs` → `dual_cubical_complex.rs`. Inner type renames: `Lattice<D>` → `CubicalComplex<D>`, `LatticeCell<D>` → `CubicalCell<D>`, `LatticeCellIterator` → `CubicalCellIterator`. Update `mod.rs` declarations.
- [ ] 3.2 Leave `src/types/gauge/gauge_field_lattice/` untouched — "lattice gauge theory" is the established physics term.
- [ ] 3.3 Move `src/extensions/hkt_lattice/` → `src/extensions/hkt_cubical_complex/`; rename the witness type accordingly. Update `src/extensions/mod.rs`.
- [ ] 3.4 Update `src/lib.rs` public exports: add `pub use crate::types::cubical_complex::{CubicalComplex, CubicalCell};` and remove `Lattice`/`LatticeCell` exports. No `pub use` aliases.
- [ ] 3.5 Move `tests/types/lattice/` → `tests/types/cubical_complex/`; rename test files in lock-step (e.g. `lattice_cell_tests.rs` → `cubical_cell_tests.rs`); update every `mod` declaration in `tests/types/mod.rs` and the new `tests/types/cubical_complex/mod.rs`; ensure `#[cfg(test)]` is preserved.
- [ ] 3.6 Update `tests/BUILD.bazel`, `BUILD.bazel`, and `Cargo.toml` (if it lists module paths) to the new directory names.
- [ ] 3.7 Create `src/traits/neighborhood.rs` defining `pub trait Neighborhood<K: ChainComplex>` with associated `type Iter<'a>: Iterator<Item = CellId> where K: 'a;` and `fn neighbors<'a>(&self, complex: &'a K, cell: CellId) -> Self::Iter<'a>;`. The `CellId` type alias lives next to the trait (likely `pub type CellId = usize;` — confirm from current code).
- [ ] 3.8 Create `src/types/neighborhood/` module with one file per strategy: `face_adjacent.rs`, `coface_adjacent.rs`, `von_neumann.rs`, `moore.rs`, `k_ring.rs`. Each strategy is a unit struct (e.g. `pub struct VonNeumann;`).
- [ ] 3.9 Implement `Neighborhood<K>` generically for `FaceAdjacent` and `CofaceAdjacent` over any `K: ChainComplex` using `boundary_matrix` / `coboundary_matrix`. Concrete iterator types (no `dyn`).
- [ ] 3.10 Implement `Neighborhood<CubicalComplex<D>>` for `VonNeumann`, `Moore`, and `KRing<const K: usize>` with grid-coordinate fast paths. Respect periodic vs open boundaries (consult `CubicalComplex::periodic`). Do NOT implement these for `SimplicialComplex`.
- [ ] 3.11 Add `Manifold::neighbors<N: Neighborhood<K>>(&self, n: N, cell: CellId) -> N::Iter<'_>` in `src/types/manifold/api/neighbors.rs`; export via `mod.rs`.
- [ ] 3.12 Write tests under `tests/traits/neighborhood_tests.rs` and `tests/types/neighborhood/{face_adjacent,coface_adjacent,von_neumann,moore,k_ring}_tests.rs`. Cover: zero-sized assertion (`assert_eq!(size_of::<VonNeumann>(), 0)`); Moore-26 on 3D open; KRing<2>-24 on 2D open; Von Neumann wrap on torus; Von Neumann omit on open boundary; `FaceAdjacent == VonNeumann` on top cells of a cube; compile-fail test (via `compile_fail` doctest or `trybuild`) that `Moore` does not implement `Neighborhood<SimplicialComplex<_>>`.
- [ ] 3.13 Register all new test files in their `tests/<dir>/mod.rs` with `#[cfg(test)]` and in `tests/BUILD.bazel`.
- [ ] 3.14 Create `examples/cubical_heat_diffusion.rs`: build a `CubicalComplex<2>::square_open(32)`, wrap in a `Manifold` with initial heat = 1.0 at the center cell and 0.0 elsewhere, run 10 steps of `CoMonad::extend` using `Moore` neighborhood and an explicit Euler heat-equation stencil, print a 32×32 ASCII heatmap each step.
- [ ] 3.15 Register the example in `Cargo.toml` (`[[example]]`) and `BUILD.bazel`.
- [ ] 3.16 Update `docs/TOPOLOGY.md`: add a "Cubical Complexes" section mirroring the existing "Simplicial Complex" section; document `Neighborhood<K>` and the strategy split (generic vs grid-only); document `SimplicialManifold` alias and the `Manifold<K, F>` generic signature.
- [ ] 3.17 Run `cargo build -p deep_causality_topology && cargo test -p deep_causality_topology && cargo run --example cubical_heat_diffusion -p deep_causality_topology`; all must succeed.
- [ ] 3.18 Run `make format && make fix && make build && make test` for the whole monorepo (three or more crates touched). All must pass.

## 4. Final verification

- [ ] 4.1 Confirm `grep -RIn "CWComplex\|Lattice\b\|LatticeCell" deep_causality_topology/src deep_causality_topology/tests` returns zero matches (sole permitted exception: the physics-named `gauge_field_lattice` submodule).
- [ ] 4.2 Confirm `grep -RIn "Box<dyn Iterator" deep_causality_topology/src/traits/ deep_causality_topology/src/types/cubical_complex/ deep_causality_topology/src/types/simplicial_complex/topology/` returns zero matches.
- [ ] 4.3 Confirm `grep -RIn "complex.coboundary_operators\[" deep_causality_topology/src/types/manifold/` returns zero matches.
- [ ] 4.4 Confirm `cargo check -p deep_causality_topology --all-features` is clean.
- [ ] 4.5 Prepare a commit message (do NOT commit — per AGENTS.md golden rule, user commits) summarizing Parts A, B, C and referencing issue #487; surface it for the user to copy.
- [ ] 4.6 Surface a follow-up issue draft for: (a) Hodge ⋆ on non-unit / irregular cubical metrics; (b) sparse `CubicalComplex` storage for high-resolution voxel grids; (c) GPU paths for cubical differential operators.
