## Context

`deep_causality_topology` already contains the algebraic primitives needed for cubical complexes: `LatticeCell<D>` encodes elementary cubes as `(position, orientation_bitmask)` and computes signed boundary chains; `Lattice<D>` implements `CWComplex` with periodic/open boundaries, boundary matrices, and Betti numbers. What is missing is integration: `Manifold` — the type that carries field data and exposes the differential / comonadic operators — is hard-wired to `SimplicialComplex<C>`. The trait that should provide the abstraction (`CWComplex`) uses `Box<dyn Iterator>`, which both blocks generic propagation through `Manifold` and violates the project's static-dispatch rule (AGENTS.md §"Static Dispatch"). Naming is also non-textbook: practitioners and the original issue (#487) speak of "cubical complexes," not "lattices."

Stakeholders:
- Service Radar team: needs cubical complexes for spatial sensor fusion (LIDAR / RF / UWB voxel grids).
- DeepCausality maintainers: must preserve simplicial behavior and existing tests bit-for-bit.

Constraints (from AGENTS.md):
- No `unsafe`, no `dyn` / trait objects, no macros in lib code, no new external dependencies, surgical diffs, tests mirror src tree, Bazel BUILD files kept in sync, one type per module.

## Goals / Non-Goals

**Goals**

- A single static-dispatch trait (`ChainComplex`) describing any CW-style complex, satisfied by both simplicial and cubical complexes.
- `Manifold` generic over `ChainComplex`, so `Manifold<CubicalComplex<D>, F>` is a first-class citizen alongside `Manifold<SimplicialComplex<C>, F>`.
- Cubical complex surfaced under textbook naming (`CubicalComplex` / `CubicalCell`).
- A neighborhood strategy trait (`Neighborhood<K>`) that is generic where it can be (chain-complex-native primitives) and grid-specific where it must be (Von Neumann / Moore / KRing).
- Existing simplicial behavior and tests untouched — no regressions, no diff outside the genericization seam.
- Worked example demonstrating voxel-grid sensor-fusion-style usage end to end.

**Non-Goals**

- Hodge ⋆ on non-unit or irregular cubical metrics (defer to a follow-up — the unit-cube case ships).
- Sparse cubical-complex storage (set-of-active-cubes). Current dense `Lattice<D>` shape suffices; sparse variant needs benchmarking before design.
- GPU paths for cubical differential operators.
- Hex, triangular, or other non-cubical CW complexes.
- Re-exporting old names (`Lattice`, `LatticeCell`, `CWComplex`) as `pub use` aliases. Clean rename, no shims.
- Behavior change in the comonad iteration order or cursor semantics.

## Decisions

### D1. Replace `CWComplex` with `ChainComplex` using GAT iterators

**Decision:** Rename the trait, replace `fn cells(&self, k) -> Box<dyn Iterator<...>>` with an associated `type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a` and a method `fn cells(&self, k: usize) -> Self::CellIter<'_>`. Add `fn coboundary_matrix(&self, k: usize) -> CsrMatrix<i8>` to the trait surface.

**Why:** `Box<dyn Iterator>` allocates and erases the iterator type, blocking inlining and monomorphization. AGENTS.md mandates static dispatch. GATs (stable since Rust 1.65) give exactly the abstraction shape we need with zero overhead. Adding `coboundary_matrix` to the trait makes Manifold's differential code generic without reaching into the concrete complex's storage.

**Alternatives considered:**
- *Keep `Box<dyn Iterator>`*: rejected — violates static-dispatch rule, hurts hot paths.
- *Return `impl Iterator` directly*: not yet stable in trait method positions in all forms we need across lifetimes; GAT is the explicit, portable shape.
- *Concrete enum iterator (`CellIterEnum<Simplex, CubicalCell<D>>`)*: rejected — closes the trait to user-defined complexes, defeats extensibility.

### D2. `Manifold<K: ChainComplex, F>` instead of `Manifold<C, D>`

**Decision:** Replace today's `Manifold<C, D> { complex: SimplicialComplex<C>, data: CausalTensor<D>, ... }` with `Manifold<K: ChainComplex, F> { complex: K, data: CausalTensor<F>, ... }`. Provide `pub type SimplicialManifold<C, F> = Manifold<SimplicialComplex<C>, F>` for ergonomics.

**Why:** The type already takes two parameters but uses them awkwardly (`C` is the simplex coordinate constraint, `D` is the field data type). Lifting `K` to the first slot expresses the actual abstraction. The alias keeps existing-style call sites short.

**Alternatives considered:**
- *Parallel `CubicalManifold` type*: rejected — duplicates the entire differential stack (`exterior_cpu`, `codifferential_cpu`, `hodge_cpu`, `laplacian_cpu`), violates AGENTS.md §"Surgical Diffs" and §"Keep the Solution Simple."
- *Trait object `Box<dyn ChainComplex>` inside `Manifold`*: rejected — `dyn` forbidden.

### D3. Move coboundary access behind the trait

**Decision:** Today `Manifold` reads `self.complex.coboundary_operators[k]` directly ([exterior_cpu.rs:42](deep_causality_topology/src/types/manifold/differential/exterior_cpu.rs#L42)). The new `ChainComplex::coboundary_matrix(k)` becomes the single access path. `SimplicialComplex` keeps its precomputed cache and returns a reference to it from the trait method. `CubicalComplex` computes `boundary_matrix(k+1).transpose()` on demand (or memoizes lazily — implementation detail, not trait contract).

**Why:** Decouples Manifold from any single complex's storage layout. Lets each complex choose its own caching policy.

**Alternatives considered:**
- *Force every complex to pre-store coboundaries*: rejected — wastes memory for complexes built from huge synthetic grids where most strata are never queried.
- *Move the cache into `Manifold`*: rejected — duplicates state already held by `SimplicialComplex`, and breaks the invariant that a complex is self-describing.

### D4. Rename `Lattice` → `CubicalComplex`, `LatticeCell` → `CubicalCell`. No back-compat aliases.

**Decision:** Move `src/types/lattice/` → `src/types/cubical_complex/`. Rename all dependent items in lock-step: `dual_lattice` → `dual_cubical_complex`, `hkt_lattice` → `hkt_cubical_complex`. The `gauge_field_lattice/` submodule retains its name (it models gauge fields *on* a lattice in the physics sense — "lattice gauge theory" is the established term and would be wrong to rename).

**Why:** AGENTS.md §"Code Conventions" and the user's explicit instruction: textbook alignment is required. Issue #487 uses "cubical complexes" verbatim. `pub use Lattice = CubicalComplex` would carry the old vocabulary into perpetuity for no benefit; per AGENTS.md preference for clean removal over back-compat shims, omit them.

**Alternatives considered:**
- *Keep `Lattice` and add `pub type CubicalComplex<const D: usize> = Lattice<D>`*: rejected by user instruction.
- *Rename and ship `pub use Lattice = CubicalComplex` for one release*: rejected — pollutes the namespace, AGENTS.md prefers clean diffs.

### D5. `Neighborhood<K>` is a static-dispatch strategy trait, generic where principled, grid-specific where honest

**Decision:** Introduce

```rust
pub trait Neighborhood<K: ChainComplex> {
    type Iter<'a>: Iterator<Item = CellId> where K: 'a;
    fn neighbors<'a>(&self, complex: &'a K, cell: CellId) -> Self::Iter<'a>;
}
```

Concrete zero-sized strategies:

| Strategy | Implemented for | Definition |
|---|---|---|
| `FaceAdjacent` | any `K: ChainComplex` | k-cells sharing a (k−1)-face with target, derived from ∂ |
| `CofaceAdjacent` | any `K: ChainComplex` | (k+1)-cells containing target as a face, derived from δ |
| `VonNeumann` | `CubicalComplex<D>` only | grid face-adjacency on top-dimensional cells (popcount/coordinate fast path) |
| `Moore` | `CubicalComplex<D>` only | all cells sharing ≥1 vertex with target (3^D − 1 around a top cube) |
| `KRing<const K: usize>` | `CubicalComplex<D>` only | Chebyshev distance ≤ K from target |

Users add custom strategies (anisotropic LIDAR cones, half-space RF) by implementing `Neighborhood<K>` for their own zero-sized types.

**Why this split:** `FaceAdjacent` / `CofaceAdjacent` are defined purely via ∂ and δ, so they extend to any chain complex without metric assumptions. `Moore` and `KRing` rely on the regular-grid coordinate structure and Chebyshev metric — there is no principled extension to arbitrary simplicial complexes. Forcing one definition over both would either (a) invent a meaningless simplicial Moore, or (b) leak grid concepts into the trait. We refuse both.

**Alternatives considered:**
- *Marker types on `Manifold<K, F, N>` baked at type level*: rejected — too rigid; users routinely want different neighborhoods per operation on the same manifold.
- *Runtime enum `enum NeighborhoodKind { VonNeumann, Moore, KRing(usize), FaceAdjacent, CofaceAdjacent }`*: rejected — runtime branch, can't express user-defined strategies, violates static-dispatch preference.
- *Two methods `extend_face` / `extend_moore` on Manifold*: rejected — non-extensible, doesn't compose.

### D6. Comonad iteration stays cursor-based; neighborhood is queried inside the user closure

**Decision:** `CoMonad::extend` continues to iterate the cursor over every cell and pass the full `Manifold` view to the user's closure (today's behavior, [hkt_manifold/mod.rs:168-193](deep_causality_topology/src/extensions/hkt_manifold/mod.rs#L168-L193)). Add a helper `Manifold::neighbors<N: Neighborhood<K>>(&self, n: N, cell: CellId) -> N::Iter<'_>` so the closure can pick a neighborhood at the point of use.

**Why:** This is the textbook cellular-automaton CoKleisli pattern. Iteration order is complex-agnostic; neighborhood choice is operation-agnostic. Decoupling them is the whole reason the comonad abstraction is useful.

### D7. `ReggeGeometry` on cubes — unit-edge case only

**Decision:** Keep `ReggeGeometry<C>` simplicial-flavored. Add a parallel `CubicalMetric<D>` (or a `MetricKind` enum at the `Manifold` level) that, for the unit-edge case, returns `1.0` for every edge length and short-circuits volume computations to integer lattice volumes. Non-uniform / scaled / curved cubical metrics are deferred.

**Why:** Sensor-fusion users overwhelmingly want unit-spacing voxel grids in v1. Building a full irregular-cubical Regge metric now would balloon the diff and isn't needed to close #487. Documenting the deferral makes the v2 contract clean.

**Alternatives considered:**
- *Genericize `ReggeGeometry` over the cell type*: doable but expensive; defer until non-uniform cubical metrics are actually needed.

## Risks / Trade-offs

- **[Risk] Manifold genericization is a wide mechanical refactor across `manifold/{api,differential,topology,geometry,covariance,constructors,getters}` — ~10 files.** → *Mitigation:* Land Part A (trait refactor) first with zero behavior change so the diff is reviewable in isolation. Part B then only changes type signatures, not logic. Keep simplicial tests unmodified; their passing is the regression gate.
- **[Risk] GAT iterators in trait methods have sharper lifetime/variance constraints than `Box<dyn Iterator>`.** Some call sites may need lifetime annotations they didn't before. → *Mitigation:* Audit during Part A; the failure mode is a compile error, not silent behavior change.
- **[Risk] Rename is a public API break for downstream crates.** `deep_causality_topology` is consumed by several monorepo crates. → *Mitigation:* Run `make build` after the rename; fix downstream import sites in the same change set. AGENTS.md tolerates this — the monorepo is co-versioned.
- **[Risk] `CubicalComplex<D>` carrying its dimension as a const generic propagates `D` into `Manifold<CubicalComplex<D>, F>` — every caller writes the dimension at the type level.** → *Mitigation:* Accept it. The alternative (runtime dimension) would erase static guarantees and break existing `Lattice<D>` users. Provide pre-baked constructors for `D = 2, 3, 4` (already present on `Lattice`).
- **[Risk] `Manifold::neighbors` returns `impl Iterator` with a borrow on `self.complex`, which can complicate borrow checking inside the comonad closure** (the closure also reads `self.data`). → *Mitigation:* The strategy returns cell IDs only — the closure then indexes `self.data` separately. No overlapping borrows of the same field.
- **[Trade-off] Moore and KRing being cubical-only means a user mixing simplicial and cubical pipelines must pick neighborhood strategies per-complex.** This is the *correct* design — the asymmetry is in the math, not the API.
- **[Trade-off] Deferring sparse cubical storage means high-resolution voxel grids (e.g., 1024³) will allocate a dense `CubicalComplex<3>` shape descriptor even if 99% of cubes are empty.** The descriptor itself is small (shape + periodicity) — what's dense is the boundary-matrix construction. For sensor fusion this may be unacceptable at scale; document it explicitly and open a follow-up.

## Migration Plan

- **Within this change set:** All three Parts (A, B, C) land in one PR with three task groups. Each task group leaves `make build && make test` green for the whole monorepo.
- **Downstream:** `deep_causality` and any other consumer of topology types updates imports in the same change. No deprecation window — the rename is atomic.
- **Rollback:** Revert the merge commit. No data migration is involved (library types only).

## Open Questions

None remaining. The neighborhood vocabulary (D5), the rename policy (D4), and the sequencing (one change set, three task groups) were settled with the user before this design was written.
