## 1. Part A — Static-dispatch ChainComplex trait (refactor + new `coboundary_matrix` method; no observable behavior change for existing callers)

> **Gate:** This stage MUST be completed, verified, signed off by the user, and committed before any task in stage 2 begins. See "Stage gates" at the end of this file.

- [ ] 1.0 Drop the `_cpu` suffix from leftover filenames and function names in `deep_causality_topology`. This is a standalone pre-cleanup, independent of the trait rename, but landed inside Stage A because it touches files Stage A and Stage B will edit anyway — doing both in one pass avoids redundant churn. Files to rename (verified via `find deep_causality_topology -name '*_cpu*'`):
  - `src/types/simplicial_complex/boundary/boundary_cpu.rs` → `boundary.rs`
  - `src/types/simplicial_complex/constructors/constructors_cpu.rs` → `constructors.rs`
  - `src/types/manifold/differential/exterior_cpu.rs` → `exterior.rs`
  - `src/types/manifold/differential/codifferential_cpu.rs` → `codifferential.rs`
  - `src/types/manifold/differential/hodge_cpu.rs` → `hodge.rs`
  - `src/types/manifold/differential/laplacian_cpu.rs` → `laplacian.rs`
  - `src/types/manifold/constructors/constructors_cpu.rs` → `constructors.rs`
  - `src/types/topology/constructors/constructors_cpu.rs` → `constructors.rs`
  - `src/types/point_cloud/constructors/constructors_cpu.rs` → `constructors.rs`
  - `src/types/graph/constructors/constructors_cpu.rs` → `constructors.rs`
  - `src/types/graph/graph_ops/graph_ops_cpu.rs` → `graph_ops.rs`
  - `src/types/hypergraph/constructors/constructors_cpu.rs` → `constructors.rs`

  Sub-steps:
  - (a) Use `git mv` for each file so history is preserved.
  - (b) Update the `pub mod constructors_cpu;` (and equivalents) in every parent `mod.rs` to match the new module name. Affected parent `mod.rs` files: `src/types/{simplicial_complex/{boundary,constructors},manifold/{differential,constructors},topology/constructors,point_cloud/constructors,graph/{constructors,graph_ops},hypergraph/constructors}/mod.rs`.
  - (c) Rename `_cpu`-suffixed function names within those files (e.g. `boundary_operator_cpu` → `boundary_operator`, `coboundary_operator_cpu` → `coboundary_operator`) and update every call site. Call sites are in the corresponding `api/*.rs` files and may be elsewhere.
  - (d) Audit gate: `find deep_causality_topology -name '*_cpu*'` returns zero matches, and `grep -RIn '_cpu' deep_causality_topology/src deep_causality_topology/tests` returns zero matches (one acceptable exception: comments that explicitly reference the historical name, if any — remove them too unless they convey load-bearing intent).
  - (e) Run `cargo build -p deep_causality_topology && cargo test -p deep_causality_topology` to confirm no behavior change. This MUST pass before the trait rename in 1.1 begins; otherwise the diff in subsequent tasks gets entangled with the cleanup.

- [ ] 1.1 Rename `src/traits/cw_complex.rs` → `src/traits/chain_complex.rs`; rename trait `CWComplex` → `ChainComplex`. Move the `Cell` marker trait out of this file into a new `src/traits/cell.rs` (per D9). Update `src/traits/mod.rs` to declare both submodules and re-export `ChainComplex` and `Cell`.
- [ ] 1.2 Replace `fn cells(&self, k: usize) -> Box<dyn Iterator<Item = Self::CellType> + '_>` with associated `type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a;` and `fn cells(&self, k: usize) -> Self::CellIter<'_>`.
- [ ] 1.3 Widen `fn boundary_matrix(&self, k: usize) -> CsrMatrix<i8>` to `fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>`. Add NEW method `fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>` to the `ChainComplex` trait contract. (Per D8.)
- [ ] 1.4 Update `src/lib.rs` re-exports: replace `pub use crate::traits::cw_complex::{CWComplex, Cell};` with `pub use crate::traits::chain_complex::ChainComplex; pub use crate::traits::cell::Cell;`.
- [ ] 1.5 Update `Lattice<D>`'s impl in `src/types/lattice/mod.rs` (the first of two existing `CWComplex` impls): (a) define `pub struct LatticeCellIter<'a, const D: usize>(LatticeCellIterator<'a, D>);` as `type CellIter<'a>` (newtype wraps the existing iterator), drop the `Box::new`; (b) change `boundary_matrix` to return `Cow::Owned(<current body>)`; (c) add `coboundary_matrix(k)` with lazy memoization — add a `coboundary_cache: RefCell<HashMap<usize, CsrMatrix<i8>>>` field to `Lattice<D>`, on first call compute `self.boundary_matrix(k + 1).into_owned().transpose()`, store and return `Cow::Owned(matrix.clone())`. No `unsafe`.
- [ ] 1.6 Update `CellComplex<C>`'s impl in `src/types/cell_complex/mod.rs` (the second extant `CWComplex` impl): (a) define a named newtype `pub struct CellComplexCellIter<'a, C>(Cloned<Iter<'a, C>>);` as `type CellIter<'a>`, drop the `Box::new`; (b) change `boundary_matrix` to return `Cow::Owned`; (c) add `coboundary_matrix(k)` returning `Cow::Owned(self.boundary_matrix(k + 1).into_owned().transpose())` with NO internal cache (per D3 — usage pattern does not justify the `RefCell` complexity).
- [ ] 1.7 Implement `ChainComplex` for `SimplicialComplex<C>` in a new file `src/types/simplicial_complex/topology/chain_complex_impl.rs`. `boundary_matrix(k)` returns `Cow::Borrowed(&self.boundary_operators[k - 1])` for `k > 0` (panics or returns empty for `k == 0`, matching today's behavior). `coboundary_matrix(k)` returns `Cow::Borrowed(&self.coboundary_operators[k])`. `cells(k)` returns a concrete iterator over the appropriate skeleton (named type, not boxed). Register the module in `src/types/simplicial_complex/topology/mod.rs`.
- [ ] 1.8 Migrate every existing call site of `CWComplex` to `ChainComplex` and update `use` statements. Affected internal files (audited): `src/types/cell_complex/{mod.rs, boundary_operator.rs}`, `src/types/lattice/{mod.rs, dual_lattice.rs}`, `src/types/gauge/gauge_field_lattice/{mod.rs, ops_actions.rs, ops_continuum.rs, ops_gauge_transform.rs, ops_gradient_flow.rs, ops_plague.rs, ops_wilson.rs}`. Audit gate: `grep -RIn 'CWComplex' src tests` returns zero matches after this task.
- [ ] 1.9 Update test files that import `CWComplex` (~25 sites with fully-qualified syntax). Files: `tests/types/cell_complex/{cell_complex_test.rs, cell_complex_boundary_tests.rs, cell_complex_homology_tests.rs}`, `tests/types/lattice/{lattice_test.rs, honeycomb_lattice_tests.rs}`, `tests/types/gauge/gauge_field_lattice/lattice_gauge_field_tests.rs`, `tests/extensions/hkt_lattice_gauge_tests.rs`. Mechanical sed pass: `CWComplex` → `ChainComplex`. Also update Cow-call sites in any test that calls `boundary_matrix` and consumes by value — use `.into_owned()` or `&*` as appropriate.
- [ ] 1.10 Replace any old `tests/traits/cw_complex_tests.rs` → `chain_complex_tests.rs`; register in `tests/traits/mod.rs` and `tests/BUILD.bazel`.
- [ ] 1.11 Add a parametric conformance test under `tests/traits/chain_complex_conformance_tests.rs` that exercises the algebraic invariants for every concrete impl (SimplicialComplex, Lattice<2/3>, CellComplex). Required assertions: (a) `boundary_matrix(k).nrows() == num_cells(k - 1) && .ncols() == num_cells(k)` for every valid `k`; (b) `coboundary_matrix(k)` equals `boundary_matrix(k + 1).transpose()` entry-by-entry; (c) for `SimplicialComplex` specifically, the returned `Cow` is `Cow::Borrowed` (`matches!(cow, Cow::Borrowed(_))`).
- [ ] 1.12 Add a static-dispatch audit test: a small Rust integration test or `compile-pass` snippet that imports `ChainComplex` and asserts `grep -RIn 'dyn Iterator' src/traits/ src/types/lattice/ src/types/cell_complex/ src/types/simplicial_complex/topology/` returns zero matches. Implementable as a `#[test]` that reads source files; failing the grep fails the test.
- [ ] 1.13 Run `cargo build -p deep_causality_topology` and `cargo test -p deep_causality_topology`; both MUST pass with no behavior change for existing callers.
- [ ] 1.14 Run `make format && make fix` and verify no new clippy warnings.
- [ ] 1.15 **Stage A gate:** prepare a stage-completion summary (changes, files touched, test evidence, deviations from spec). Surface it to the user. Wait for explicit written sign-off. Then prepare a commit message for the user to commit. Do NOT advance to stage 2 until the commit has landed.

## 2. Part B — Genericize Manifold over ChainComplex

> **Gate:** Stage 1 (Part A) MUST be signed off and committed before any task in stage 2 begins.

- [ ] 2.1 In `src/types/manifold/mod.rs`, change the struct from `Manifold<C, D>` with `complex: SimplicialComplex<C>` to `Manifold<K: ChainComplex, F>` with `complex: K`, `data: CausalTensor<F>`, `metric: Option<ReggeGeometry<K>>` (or `Option<Box<...>>` only if forced — prefer trait-generic metric; see 2.5), `cursor: usize`.
- [ ] 2.2 Add public type aliases in `src/types/manifold/mod.rs`: `pub type SimplicialManifold<C, F> = Manifold<SimplicialComplex<C>, F>;` AND `pub type SimplicialManifoldWitness<C> = ManifoldWitness<SimplicialComplex<C>>;`. Re-export both from `lib.rs`. The witness alias is required to smooth the migration of example files that name `ManifoldWitness<C>` directly (multiple sites under `examples/mathematics_examples/`).
- [ ] 2.3 Update every file under `src/types/manifold/{api,constructors,covariance,display,geometry,getters,utils,differential,topology}/` to take `K: ChainComplex` instead of the concrete `SimplicialComplex<C>`. Mechanical sed-style edits — no logic change.
- [ ] 2.4 In `src/types/manifold/differential/exterior_cpu.rs`, replace `&self.complex.coboundary_operators[k]` with `&*self.complex.coboundary_matrix(k)` (the `Cow::Borrowed` path through Part A's trait returns a zero-copy borrow on `SimplicialComplex`). Repeat for `codifferential_cpu.rs`, `hodge_cpu.rs`, `laplacian_cpu.rs` — all reads of pre-cached boundary/coboundary go through trait methods. MUST NOT call `.into_owned()` on the read path; doing so reintroduces the clone that Cow exists to avoid.
- [ ] 2.5 Resolve `ReggeGeometry<K>` genericization: either (a) keep `ReggeGeometry` simplicial and introduce a `MetricKind<K>` enum at the `Manifold` level holding `Simplicial(ReggeGeometry<...>)` or `Cubical(CubicalMetric<D>)`, or (b) genericize `ReggeGeometry<K: ChainComplex>` directly. Pick (a) — it isolates the diff. Create `src/types/cubical_metric/mod.rs` carrying the unit-edge cubical metric (constant edge length `1.0`).
- [ ] 2.6 Update `src/extensions/hkt_manifold/mod.rs` to re-parameterize `ManifoldWitness` and the `Functor`/`CoMonad` impls over `K: ChainComplex` instead of `C: Satisfies<NoConstraint>`. Iteration order and cursor semantics MUST remain identical.
- [ ] 2.7 Update every existing import/use of `Manifold<C, D>` to `SimplicialManifold<C, D>` across `deep_causality_topology` and any downstream monorepo crate (`grep -RIn "Manifold<" .`).
- [ ] 2.8 Update test files under `tests/types/manifold/` to use `SimplicialManifold`; do not rename test files (the underlying type is still being exercised).
- [ ] 2.9 Add an assertion test that `grep -RIn "complex.coboundary_operators" src/types/manifold/` returns zero matches — encoded as a Bazel `genrule` test or a Rust `build.rs` guard if the repo already has that pattern; otherwise document the check in `tests/types/manifold/no_direct_field_access_tests.rs` as a `cargo test` integration that scans the source.
- [ ] 2.10 Run `cargo build -p deep_causality_topology && cargo test -p deep_causality_topology` — every pre-existing simplicial test must pass unchanged.
- [ ] 2.11 Run `make build` to confirm no downstream monorepo crate breaks (this is where the import migration in 2.7 is verified end-to-end).
- [ ] 2.12 Run `make format && make fix`.
- [ ] 2.13 **Stage B gate:** prepare a stage-completion summary including evidence that no `CsrMatrix` clones were introduced on the Manifold differential read path (point at the `&*cow` call sites). Surface to the user. Wait for explicit written sign-off. Then prepare a commit message for the user to commit. Do NOT advance to stage 3 until the commit has landed.

## 3. Part C — Cubical surfacing, Neighborhood strategy, and example

> **Gate:** Stages 1 (Part A) and 2 (Part B) MUST both be signed off and committed before any task in stage 3 begins.

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
- [ ] 3.19 **Stage C gate:** prepare a stage-completion summary covering the rename ripples, the new neighborhood strategies, the example, and the final monorepo build/test results. Surface to the user. Wait for explicit written sign-off. Then prepare a commit message for the user to commit.

## 4. Final verification

> **Gate:** All three stages (A, B, C) MUST be signed off and committed before final verification begins.

- [ ] 4.1 Confirm `grep -RIn "CWComplex\|Lattice\b\|LatticeCell" deep_causality_topology/src deep_causality_topology/tests` returns zero matches (sole permitted exception: the physics-named `gauge_field_lattice` submodule).
- [ ] 4.2 Confirm `grep -RIn "Box<dyn Iterator" deep_causality_topology/src/traits/ deep_causality_topology/src/types/cubical_complex/ deep_causality_topology/src/types/simplicial_complex/topology/` returns zero matches.
- [ ] 4.3 Confirm `grep -RIn "complex.coboundary_operators\[" deep_causality_topology/src/types/manifold/` returns zero matches.
- [ ] 4.4 Confirm `cargo check -p deep_causality_topology --all-features` is clean.
- [ ] 4.5 Surface a follow-up issue draft for: (a) Hodge ⋆ on non-unit / irregular cubical metrics; (b) sparse `CubicalComplex` storage for high-resolution voxel grids; (c) GPU paths for cubical differential operators.

## 5. Migration (post-refactor, last task in the change set)

> **Gate:** Stages 1, 2, and 3 MUST all be signed off and committed before any task in this section begins. This is the final stage; it closes the change set.

- [ ] 5.1 Open `openspec/changes/add-cubical-complexes/migration.md`. Walk every entry in section 2 ("Sites to migrate") top-to-bottom, applying the rules in section 1 ("Migration rules") mechanically. Check off the ⬜ box for each file as it is updated.
- [ ] 5.2 Run the verification protocol in `migration.md` section 3: `cargo build --workspace --all-targets`, `cargo test --workspace --all-targets`, `make format && make fix`, the example-run pass, and the final grep audit. Check off each step in `migration.md` as it passes.
- [ ] 5.3 If the grep audit in step 5.2 surfaces a site not listed in `migration.md` section 2, append it to that section before fixing it. The file must end the stage as a complete record of every breaking-change call site touched.
- [ ] 5.4 **Migration gate:** prepare a stage-completion summary covering every file in `migration.md` section 2, the verification evidence, and any late-discovered sites. Surface to the user. Wait for explicit written sign-off. Then prepare a commit message for the user to commit. After the commit lands, the change set is closed and ready to archive via `openspec` archival tooling.

## Stage gates (binding)

Each stage in this change set is gated. Tasks in a later stage MUST NOT begin until the prior stage is complete, verified, signed off, and committed.

**Per-stage completion criteria (all required):**
- Every task in the stage is checked off in this file.
- `cargo build -p deep_causality_topology` and `cargo test -p deep_causality_topology` both green.
- `make format && make fix` clean; no new clippy warnings.
- Every scenario in the stage's spec file(s) is verified (either by an existing test or by a new test added in the stage).

**Sign-off protocol:**
- The agent presents a stage-completion summary: a concise list of files changed, the verification evidence (test names + pass status, grep audits), and any deviations from spec with reasoning.
- The user reviews and either (a) approves explicitly in writing ("approved" / "looks good" / "go ahead with stage N+1") or (b) requests revisions.
- Implicit approval, silence, or hedged language does NOT count as sign-off.

**Commit protocol:**
- Per AGENTS.md §"Golden Rules" rule 1, the agent NEVER commits.
- After sign-off, the agent prepares a commit message (HEREDOC form, with `Co-Authored-By` footer) and presents it.
- The user runs the commit. The next stage starts only after the commit lands.

**Failed-review handling:**
- If the user requests revisions, the stage returns to in-progress. The gate does not advance until the user re-approves a corrected stage-completion summary.
