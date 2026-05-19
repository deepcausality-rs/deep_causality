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

**Decision:** Rename the trait, replace `fn cells(&self, k) -> Box<dyn Iterator<...>>` with an associated `type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a` and a method `fn cells(&self, k: usize) -> Self::CellIter<'_>`. Add `fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>` to the trait surface and widen `fn boundary_matrix(&self, k: usize)` to the same `Cow<'_, CsrMatrix<i8>>` return type (see D8).

**Why:** `Box<dyn Iterator>` allocates and erases the iterator type, blocking inlining and monomorphization. AGENTS.md mandates static dispatch. GATs (stable since Rust 1.65, project MSRV is 1.90) give exactly the abstraction shape we need with zero overhead. Adding `coboundary_matrix` to the trait makes Manifold's differential code generic without reaching into the concrete complex's storage. The `coboundary_matrix` method is a brand-new addition to the trait, not a consolidation of an existing one — today's `CWComplex` exposes only `boundary_matrix`.

**Alternatives considered:**
- *Keep `Box<dyn Iterator>`*: rejected — violates static-dispatch rule, hurts hot paths.
- *Return `impl Iterator` directly via RPITIT*: available on MSRV 1.90 but cannot constrain `Send`/`Sync` on the associated iterator type or expose it as a named type in other generic contexts; GAT is the more portable shape.
- *Concrete enum iterator (`CellIterEnum<Simplex, CubicalCell<D>>`)*: rejected — closes the trait to user-defined complexes, defeats extensibility.

### D2. `Manifold<K: ChainComplex, F>` instead of `Manifold<C, D>`

**Decision:** Replace today's `Manifold<C, D> { complex: SimplicialComplex<C>, data: CausalTensor<D>, ... }` with `Manifold<K: ChainComplex, F> { complex: K, data: CausalTensor<F>, ... }`. Provide `pub type SimplicialManifold<C, F> = Manifold<SimplicialComplex<C>, F>` for ergonomics.

**Why:** The type already takes two parameters but uses them awkwardly (`C` is the simplex coordinate constraint, `D` is the field data type). Lifting `K` to the first slot expresses the actual abstraction. The alias keeps existing-style call sites short.

**Alternatives considered:**
- *Parallel `CubicalManifold` type*: rejected — duplicates the entire differential stack (`exterior_cpu`, `codifferential_cpu`, `hodge_cpu`, `laplacian_cpu`), violates AGENTS.md §"Surgical Diffs" and §"Keep the Solution Simple."
- *Trait object `Box<dyn ChainComplex>` inside `Manifold`*: rejected — `dyn` forbidden.

### D3. Move coboundary access behind the trait

**Decision:** Today `Manifold` reads `self.complex.coboundary_operators[k]` directly ([exterior_cpu.rs:42](deep_causality_topology/src/types/manifold/differential/exterior_cpu.rs#L42)). The new `ChainComplex::coboundary_matrix(k)` becomes the single access path, returning `Cow<'_, CsrMatrix<i8>>` (see D8). `SimplicialComplex` keeps its precomputed cache and vends a `Cow::Borrowed` from the trait method. `CubicalComplex` lazily memoizes `boundary_matrix(k+1).transpose()` inside a `RefCell<HashMap<usize, CsrMatrix<i8>>>` and vends a `Cow::Owned` of the cached matrix on first call (subsequent calls also return `Cow::Owned` clones — `Cow` cannot borrow through `RefCell`; the clone here is from a known-built matrix, not a recomputation). `CellComplex` recomputes on each call and vends `Cow::Owned` — its usage is infrequent enough that lazy memoization is not justified.

**Why:** Decouples Manifold from any single complex's storage layout. Lets each complex choose its own caching policy. The `Cow` return shape (D8) lets cache-rich impls avoid the `clone()` that a by-value trait return would force.

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

### D8. Trait matrix-return type is `Cow<'_, CsrMatrix<i8>>`

**Decision:** Both `boundary_matrix(k)` and `coboundary_matrix(k)` on the `ChainComplex` trait return `std::borrow::Cow<'_, CsrMatrix<i8>>`. Cache-rich implementors (`SimplicialComplex`) vend `Cow::Borrowed(&self.boundary_operators[k - 1])` — zero copy. Compute-on-demand implementors (`Lattice`, `CellComplex`) vend `Cow::Owned(matrix)`. Call sites that need to consume the matrix call `.into_owned()`; call sites that only need to read call `&*matrix` or `matrix.as_ref()`.

**Why:** Today's signature is by value (`-> CsrMatrix<i8>`). Today's `SimplicialComplex` fast path is by reference (`boundary_operator_cpu -> Result<&CsrMatrix, _>`). `Manifold`'s current differential code reads through the reference and never clones. If Part B routes those reads through a by-value trait method, every exterior-derivative / codifferential / Hodge / Laplacian call on a `SimplicialManifold` clones a `CsrMatrix` per grade — a silent perf regression that "tests still pass" would never catch. `Cow` lets the cache-rich path stay zero-copy without forcing compute-on-demand impls to manufacture a `'static` reference they don't have.

**Alternatives considered:**
- *By value (`-> CsrMatrix<i8>`)*: rejected — `SimplicialComplex` would clone its cached matrices on every call, regressing hot paths.
- *By reference (`-> &CsrMatrix<i8>`)*: rejected — forces `Lattice` and `CellComplex` to memoize internally just to vend a reference, even when the caller would have been fine with a fresh matrix. Spreads `RefCell` and complicates the trait's borrow story.
- *Result-wrapped (`-> Result<Cow<'_, CsrMatrix<i8>>, TopologyError>`)*: rejected for Part A — the existing trait is infallible and existing tests rely on that. Out-of-range `k` is a programming error; impls panic (consistent with today's `boundary_operators[k - 1]` indexing behavior) or return an empty matrix as `Lattice` does today.

### D9. Co-locate `Cell` trait in its own file

**Decision:** Move the `Cell` marker trait out of `traits/cw_complex.rs` (renamed to `traits/chain_complex.rs`) and into a new `traits/cell.rs`. Re-export both from `traits/mod.rs`.

**Why:** The `Cell` trait is conceptually distinct from `ChainComplex` (a `Cell` is an atomic unit; a `ChainComplex` is the algebra over many cells). They are bundled today because both fit on one page. While we are touching the file anyway, splitting them aligns with the project rule "one type, one Rust module" (AGENTS.md §"One type, one Rust module"). Cost is one extra file; benefit is the file structure now matches the conceptual structure, and grep-for-trait stops returning two hits.

**Alternatives considered:**
- *Leave `Cell` in the renamed `chain_complex.rs`*: low-cost status quo but inconsistent with the project convention and slightly misleading to readers expecting one trait per file.

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

- **Within this change set:** All three Parts (A, B, C) land sequentially under stage gates. Each task group leaves `make build && make test` green for the whole monorepo before the next stage begins.
- **Downstream:** `deep_causality` and any other consumer of topology types updates imports in the same change. No deprecation window — the rename is atomic.
- **Rollback:** Revert the relevant commit(s). Because each stage lands as its own commit (per the gate protocol below), Part A and Part B can be reverted independently if needed. No data migration is involved (library types only).

## Stage gates (binding)

The proposal establishes a binding gate between stages. Repeated here so design and tasks share the same contract:

1. **Per-stage completion criteria:** all checkboxes in the stage's task group complete; `cargo build -p deep_causality_topology` + `cargo test -p deep_causality_topology` green; `make format && make fix` clean; spec scenarios for that stage verified.
2. **Sign-off:** the agent presents a stage-completion summary (changes, verification evidence, deviations). The user reviews and either approves explicitly (in writing) or requests revisions. Implicit approval does not count.
3. **Commit:** per AGENTS.md §"Golden Rules" rule 1, the agent NEVER commits. After user sign-off, the agent prepares a commit message and the user commits. Only after the commit lands does the next stage begin.
4. **Failed review:** the stage returns to in-progress; the gate does not advance until the user re-approves.

## Deviations log (implementation reconciliation)

This section records intentional differences between the as-written design / spec and the as-implemented code. Each entry is paired with its Stage of origin and the spec text that was reconciled.

### Stage A

- **D1-actual.** `Lattice<D>` coboundary cache uses `std::sync::Mutex<HashMap<...>>` instead of `RefCell<HashMap<...>>`. `RefCell` would have made `Lattice<D>: !Sync`, breaking the existing `Arc<Lattice<D>>` consumers in `gauge_field_lattice`. Spec text in `chain-complex-trait/spec.md` § "Lattice satisfies the trait via lazy-memoized Cow::Owned" updated to reflect `Mutex`.
- **D2-actual.** `Cell` was not implemented for `Simplex` in the pre-Stage-A tree (because the original `CWComplex` impl for `SimplicialComplex` did not exist). Stage A adds `impl Cell for Simplex` with the standard signed-face boundary. Recorded as a requirement in `simplicial-complex/spec.md` § "Simplex implements the Cell marker trait".
- **D3-actual.** `SimplicialComplex::betti_number` was a new method (the trait required it, the type didn't have it). Implementation lifts the boundary `CsrMatrix<i8>` to `f64` and computes rank via SVD with tolerance `1e-5`, mirroring `CellComplex::rank_of_matrix`. Recorded in `simplicial-complex/spec.md` § "SimplicialComplex::betti_number is computed via SVD-based rank".
- **D4-actual.** `_cpu` filename suffix dropped from leftover files; function-name suffix renamed to `_impl` (kept the api/impl two-layer wrapper pattern at the user's direction). 12 file renames, 14 function-name renames. Recorded in `simplicial-complex/spec.md` scenario notes (`boundary_operator_cpu` → `boundary_operator_impl`).

### Stage B

- **D5-actual.** `ManifoldWitness<C>` was NOT re-parameterized over `K: ChainComplex` (despite original Stage B task wording). Its `Functor`/`Monad`/`Applicative`/`CoMonad` impl bodies assume simplicial-specific bounds (`Default::default()` on the complex, `Clone` on `K::Metric`, etc.). Lifting `K` requires a fresh HKT-machinery design that is orthogonal to the `Manifold` struct genericization. Stage B ships only the alias `SimplicialManifoldWitness<C> = ManifoldWitness<C>` as the textbook entry point. A separate `GenericManifoldWitness<K>` is queued for Stage C (tasks.md task 3.11a). Recorded in `manifold/spec.md` § "Comonad iteration is unchanged …".
- **D6-actual.** `manifold/differential/hodge.rs` and `manifold/differential/laplacian.rs` were left unmodified by Stage B. On audit, neither reads the boundary/coboundary cache directly — they call the already-routed `exterior_derivative` / `codifferential` methods and the `hodge_star_operators` field (which is simplicial-only and not part of `ChainComplex`). Adding Hodge ⋆ to the trait is design D7's explicit "deferred to follow-up" item, so the simplicial-only restriction on `hodge.rs` / `laplacian.rs` is correct. Recorded in `manifold/spec.md` § "Differential operators read the complex through the trait" (the Hodge ⋆ scope paragraph).
- **D7-actual.** `manifold/utils/utils_manifold.rs` (`is_oriented`, `has_boundary`) was migrated through `ChainComplex::boundary_matrix` even though the file was not in Stage B task 2.5's list. The audit grep would otherwise have flagged these helpers, requiring an exception clause; routing them through the trait keeps the audit rule uniform. Recorded in `manifold/spec.md` § "Differential operators read the complex through the trait".

## Open Questions

None remaining. Neighborhood vocabulary (D5), rename policy (D4), sequencing, the trait matrix-return type (D8), and the `Cell` trait co-location (D9) are all settled. The deviations above are reconciled in the spec files and recorded here for traceability; they do not introduce new open questions.
