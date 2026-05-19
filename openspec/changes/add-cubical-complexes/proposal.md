## Why

GitHub issue #487 asks for cubical complexes to support efficient spatial sensor fusion (LIDAR / RF / UWB / voxel grids), driven by Service Radar's adoption of DeepCausality for topological network analysis. The substrate already exists in `deep_causality_topology` as `Lattice<const D: usize>` with `LatticeCell<D>`, periodic/open boundaries, boundary matrices and Betti numbers — but it is **inaccessible from `Manifold`**, carries non-textbook naming, and is plumbed through a chain-complex trait (`CWComplex`) that violates the project's static-dispatch rule. As a result, no user can put field data on a cubical grid and run the differential/comonadic machinery the simplicial path already has. This change unblocks #487 by completing the missing plumbing, not by inventing a new primitive.

## What Changes

**Part A — Static-dispatch chain-complex trait (pure refactor, no behavior change)**

- **BREAKING (internal trait surface only):** Rename `CWComplex` → `ChainComplex` to align with algebraic-topology textbook terminology.
- **BREAKING:** Replace `fn cells(&self, k) -> Box<dyn Iterator<...>>` with a GAT-backed `type CellIter<'a>` returning `impl Iterator`. Eliminates `dyn` per AGENTS.md static-dispatch rule.
- Add `fn coboundary_matrix(&self, k: usize) -> CsrMatrix<i8>` to the trait contract so the differential stack can be written generically.
- Implement `ChainComplex` for `SimplicialComplex` (today it has equivalent methods but does not wear the trait).
- Update existing `Lattice<D>` impl to the new trait surface.

**Part B — Genericize `Manifold` over `ChainComplex`**

- **BREAKING (public type signature):** `Manifold<C, D>` (where the complex was hard-coded to `SimplicialComplex<C>`) becomes `Manifold<K: ChainComplex, F>` where `K` is the underlying complex and `F` is field data.
- Add type alias `pub type SimplicialManifold<C, F> = Manifold<SimplicialComplex<C>, F>` for ergonomic continuity; existing call sites that used `Manifold<C, D>` migrate to the alias.
- Move pre-computed coboundary operators out of `SimplicialComplex` storage or expose them through the trait, so `manifold/differential/*` (exterior derivative, codifferential, Hodge ⋆, Laplacian) read through `K: ChainComplex` instead of reaching into `self.complex.coboundary_operators[k]`.
- Re-implement the `ManifoldWitness` HKT extensions (`Functor`/`CoMonad` for `Manifold`) against the generic complex parameter.
- `ReggeGeometry`: keep the simplicial implementation; add a trivial cubical metric for `Lattice<D>` (unit edges / scalar spacing) so `Manifold<Lattice<D>, F>` is constructible. Non-uniform cubical metrics are deferred.

**Part C — Cubical-complex surfacing and neighborhood strategy (the user-visible feature)**

- **BREAKING (rename):** Rename `Lattice<const D: usize>` → `CubicalComplex<const D: usize>` and `LatticeCell<D>` → `CubicalCell<D>` to match textbook usage and the issue's terminology. The `types/lattice/` directory moves to `types/cubical_complex/`. Dependent modules (`gauge_field_lattice`, `dual_lattice`, `hkt_lattice`, `specialized`) are renamed in lock-step. No `pub use` aliases for the old names — the project prefers clean removal over back-compat shims.
- Introduce `Neighborhood<K>` strategy trait with a single method returning `impl Iterator<Item = CellId>`. Zero-sized concrete strategies, all static dispatch:
  - Generic over any `ChainComplex`: `FaceAdjacent` (via ∂), `CofaceAdjacent` (via δ).
  - `CubicalComplex<D>` only: `VonNeumann`, `Moore`, `KRing<const K: usize>`.
  - Users can implement `Neighborhood<K>` for their own strategies (anisotropic LIDAR cones, half-space RF) without forking the crate.
- Add a `Manifold` helper `fn neighbors<N: Neighborhood<K>>(&self, n: N, cell: CellId) -> impl Iterator<...>` so the comonad's `extend` closure can query neighborhoods uniformly.
- Add `examples/cubical_heat_diffusion.rs` showing voxel-grid construction, boundary, and Moore-neighborhood heat diffusion via `CoMonad::extend`.

**Not in scope (deferred to follow-up issues)**

- Hodge ⋆ on non-unit / non-regular cubes (irregular cubical metrics).
- Sparse cubical complex (set-of-active-cubes) vs the current implicit-dense `CubicalComplex<D>`. Sensor fusion likely wants sparse; needs benchmarking first.
- Hex / triangular / other CW complex kinds.
- GPU paths for cubical Hodge / Laplacian (current `*_cpu.rs` naming already implies a future GPU split).

## Capabilities

### New Capabilities

- `chain-complex-trait`: Static-dispatch trait describing any CW complex (simplicial, cubical, or user-defined) in terms of cells, boundary, coboundary, and Betti numbers. Replaces the dyn-based `CWComplex` trait.
- `cubical-complex`: First-class cubical complex type (renamed from `Lattice`) with elementary cubes, periodic/open boundaries, boundary/coboundary matrices, and dense + future-sparse storage. Implements `chain-complex-trait`.
- `neighborhood-strategy`: Static-dispatch trait family for cell neighborhood queries. Provides chain-complex-generic primitives (`FaceAdjacent`, `CofaceAdjacent`) and cubical-only strategies (`VonNeumann`, `Moore`, `KRing`). Consumed by the comonadic `extend` operation.

### Modified Capabilities

- `manifold`: Becomes generic over any `chain-complex-trait` implementor instead of hard-wired to `SimplicialComplex`. Differential operators (exterior derivative, codifferential, Hodge ⋆, Laplacian) and the `CoMonad` impl are reworked to read through the trait. Existing simplicial behavior preserved via `SimplicialManifold<C, F>` alias.
- `simplicial-complex`: Implements `chain-complex-trait`. Pre-cached coboundary operators are exposed through the trait surface instead of via direct field access from `Manifold`.

## Impact

**Affected crate:** `deep_causality_topology` (sole crate touched).

**Affected modules:**
- `src/traits/cw_complex.rs` — trait redesign and rename.
- `src/traits/mod.rs` — re-export rename.
- `src/types/lattice/` → `src/types/cubical_complex/` — directory move + type rename + downstream module renames (`gauge_field_lattice`, `dual_lattice`, `specialized`).
- `src/types/manifold/**` — genericization (largest mechanical diff: ~10 files under `api/`, `differential/`, `topology/`, `geometry/`, `covariance/`, `constructors/`).
- `src/types/simplicial_complex/topology/` — `ChainComplex` impl.
- `src/extensions/hkt_manifold/` — re-parameterize witness over `K: ChainComplex`.
- `src/extensions/hkt_lattice/` → `src/extensions/hkt_cubical_complex/`.
- `src/types/` — new `neighborhood/` module for the strategy trait and concrete strategies.
- `tests/**` — mirror all structural moves; update `tests/BUILD.bazel`.
- `examples/` — new `cubical_heat_diffusion.rs`.
- `BUILD.bazel`, `Cargo.toml` — no new external dependencies; only module path updates.

**External API impact:**
- Public renames: `Lattice<D>` → `CubicalComplex<D>`, `LatticeCell<D>` → `CubicalCell<D>`, `CWComplex` → `ChainComplex`. No `pub use` aliases.
- Public type signature change: `Manifold<C, D>` → `Manifold<K, F>`. `SimplicialManifold<C, F>` alias provided.
- Downstream crates depending on `deep_causality_topology`'s renamed types must update imports.

**Constraints honored:**
- No `unsafe` introduced.
- No `dyn` / trait objects; static dispatch throughout.
- No macros in lib code.
- No new external dependencies.
- Tests mirror src structure and are registered in `tests/mod.rs` + `tests/BUILD.bazel`.
- No `pub use` back-compat shims for renamed items (per AGENTS.md preference for clean diffs).

**Sequencing inside the change** (single OpenSpec change, three task groups in `tasks.md`):
1. Part A (chain-complex trait refactor) — must land first; no behavior change.
2. Part B (Manifold genericization) — depends on Part A; existing simplicial tests must continue to pass unchanged.
3. Part C (cubical surfacing + neighborhood strategies + example) — depends on Part B; this is the change that actually closes #487.
