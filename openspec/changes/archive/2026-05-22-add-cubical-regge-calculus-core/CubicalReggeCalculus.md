# Cubical Regge Calculus — forward-looking design note

**Status:** Forward-looking. Stage C of `add-cubical-complexes` shipped the type scaffold (`CubicalReggeGeometry<D>` with 4-level edge-length storage and Lorentzian-axis flags) but not the derived geometric machinery. This note specifies what the topology crate would need to implement the full Cubical Regge calculus, contrasts it with the standard simplicial Causal Dynamical Triangulations (CDT) approach, and discusses the new capabilities it unlocks — especially in combination with the HKT machinery that already makes the crate's topology / algebra / tensor layers uniformly composable.

The note is intentionally concrete: each phase below lists the exact methods, fields, and source-file additions a follow-up change set would need. The order is dependency-respecting — each phase builds on the previous one.

---

## 0. Where we are now

After `add-cubical-complexes` (issue #487):

- **`LatticeComplex<D>`** (canonical name; `CubicalComplex<D>` is a type alias) implements `ChainComplex`. It stores `shape: [usize; D]`, `periodic: [bool; D]`, and a `Mutex`-backed coboundary cache. Cells are enumerated on demand by `LatticeCellIterator`, encoding each elementary cube as `(position, orientation_bitmask)`.
- **`CubicalReggeGeometry<D>`** is the `ChainComplex::Metric` associated type for `LatticeComplex<D>`. Its fields are sufficient inputs for the full Regge construction:
  - Edge-length representation at four levels of uniformity (`UnitEdge`, `Uniform { length }`, `PerAxis { lengths: [f64; D] }`, `PerEdge { lengths: Vec<f64> }`).
  - Optional `timelike_axes: Option<[bool; D]>` for Lorentzian lattices.
- **`Manifold<LatticeComplex<D>, F>`** wraps complex + field data + `Option<CubicalReggeGeometry<D>>` + cursor. Constructors `from_cubical` / `from_cubical_with_metric` ship.
- **Differential operators** in `manifold/differential/` (`exterior`, `codifferential`, `hodge`, `laplacian`) are currently implemented for `Manifold<SimplicialComplex<C>, D>` only. The exterior/codifferential paths route through `ChainComplex::boundary_matrix` / `coboundary_matrix` and are *almost* generic — they just need a metric layer (Hodge ⋆) that works on lattices.
- **`Neighborhood<K>`** strategies are in place: `FaceAdjacent`, `CofaceAdjacent` (generic), `VonNeumann`, `Moore`, `KRing<const K>` (`LatticeComplex<D>` only).

What's missing for the full Cubical Regge picture is the **geometric derivation layer**: cell volumes, hinge enumeration with dihedral angles, deficit angles, the Hodge ⋆ on lattice complexes, the Regge action, and the gradient of the action with respect to edge lengths. The type already carries the inputs for all of it.

---

## 1. The mathematical construction (what we are building)

Regge calculus replaces a smooth Riemannian (or Lorentzian) manifold with a piecewise-flat cellular complex. Each top-dimensional cell is rigid — its metric is determined by its edge lengths — and curvature concentrates on the codimension-2 cells (called *hinges* or *bones*).

In the standard **simplicial** Regge calculus (Tullio Regge, *Nuovo Cimento* 1961):

- Cells are k-simplices. A 4D triangulation has 4-simplices as top cells, with 2-simplices (triangles) as hinges.
- Edge lengths determine the metric inside each simplex (Cayley-Menger).
- Around each hinge, the dihedral angles of the incident top simplices sum to less than 2π in curved space; the deficit `ε(h) = 2π − Σ θᵢ(h)` measures intrinsic curvature concentrated at the hinge.
- The Einstein-Hilbert action is `S_EH = (1/(8πG)) · Σ_h V(h) · ε(h)`, where `V(h)` is the (D−2)-volume of the hinge.

The **cubical** version (sometimes called *cubical Regge calculus* or *hypercubic Regge calculus*) keeps the same construction with cubical cells. In a D-dimensional lattice complex:

- Top cells are D-cubes. Hinges are (D−2)-cells: vertices in 2D, edges in 3D, squares in 4D.
- Each top cube has 2D faces and 4·(D choose 2) edges. Its metric in the per-edge case is the Gram matrix of the D edge vectors meeting at any vertex of the cube; in the per-axis case the metric is diagonal with entries `lengths[i]²`.
- Dihedral angles at a hinge: the angle between the two faces of a top cube that share the hinge. On the unit-edge lattice every dihedral angle is π/2; with edge-length variation the angles deform.
- Deficit angles, Regge action: identical formulas to the simplicial case, just with cubical hinges and dihedral angles substituted in.
- The unit-edge ℤᴰ lattice is flat: each hinge has 2(D−1) incident top cubes (each contributing π/2 to the dihedral sum in the appropriate normal plane) — concretely, in 4D each 2-cell hinge has 4 incident 4-cubes contributing π/2 each, totalling 2π, so ε = 0 everywhere.

Mathematically the constructions are completely parallel. The differences are practical (how easy is each step to compute on the respective complex type) and topological (which manifolds each complex type can represent).

---

## 2. Differences from simplicial CDT

Causal Dynamical Triangulations (Ambjørn–Jurkiewicz–Loll, 1998–) is a specific quantum-gravity research program built on simplicial Regge calculus, with one critical extra ingredient: a **causal foliation** that enforces global hyperbolicity.

| Aspect | Simplicial CDT (status quo in lattice quantum gravity) | Cubical Regge calculus (proposed) |
|---|---|---|
| **Cells** | k-simplices; each top simplex has k+1 vertices and (k+1)(k+2)/2 edges | k-cubes; each top cube has 2ᵏ vertices and k·2^(k−1) edges |
| **Topology flexibility** | Any triangulable manifold can be represented | Only manifolds that admit a regular cubical decomposition (the unit ℤᴰ-grid topology is fixed unless the cubical complex is augmented with cubical subdivisions) |
| **Foliation / causal structure** | Imposed explicitly: every spatial slice is a (D−1)-triangulation, slices are stitched by timelike edges with prescribed length ratio α | Implicit in the product structure: each axis is either spacelike or timelike (the `timelike_axes` flag); slicing is free along any timelike axis |
| **Hinges in 4D** | Triangles. Each 2-simplex hinge is incident to a variable number of 4-simplices (typically ~5 on average in 4D CDT runs) | Squares. Each 2-cell hinge is incident to exactly 4 4-cubes on a regular grid — count is fixed by the lattice topology |
| **Dihedral angles** | Require simplex-by-simplex computation from edge lengths (Cayley-Menger + arccosines); ~6 angles per simplex per hinge in 4D | On a unit-edge grid all angles are π/2; deviations are perturbations from this baseline — much simpler closed-form expressions |
| **Manifold metric per cell** | Cayley-Menger Gram matrix from edge lengths; signature can vary cell to cell | Diagonal Gram matrix in the per-axis case (constant signature); per-edge case has full Gram matrix but with axis-aligned eigenframe |
| **Sum-over-histories** | Monte Carlo over the *set of triangulations* (vertices, simplices, edge-length assignments). Markov moves include Pachner moves that change topology of the triangulation. | Monte Carlo over edge-length assignments at *fixed* lattice topology. No analog of Pachner moves; the grid topology is invariant. |
| **What's varied** | Both the connectivity (which simplices share which faces) and edge lengths | Only edge lengths. Connectivity is fixed by `LatticeComplex<D>`. |
| **Emergent dimension** | CDT finds an effective spectral dimension of ~4 in the IR and ~2 in the UV — a non-trivial dynamical result | Less clear; the fixed cubical topology may forbid the dimensional reduction CDT observes. An open research question. |

**Net characterization:** simplicial CDT is *path-integral over geometries with topology change allowed within a global foliation*. Cubical Regge calculus is *path-integral over edge-length fluctuations on a fixed cubical topology, with causal structure built into the per-axis signature*.

Each is suited to different questions:
- **CDT-style work** is the right tool when emergent dimensionality, topology fluctuations, or sum-over-topologies effects are central.
- **Cubical Regge** is the right tool when you want to study metric fluctuations on a fixed lattice topology — for example: lattice quantum gravity coupled to lattice gauge fields (where the gauge field lives natively on lattice links and benefits from the regular grid), backreaction studies, anisotropic spacetime studies (where the axis-aligned structure makes anisotropy a first-class concept), and discrete-form numerical PDE methods on Cartesian grids.

The two approaches are complementary, not competitors.

---

## 3. Implementation phases

Each phase is independently shippable and has a verifiable algebraic invariant (a property test). The phases are ordered by data dependency: Phase Rₙ assumes Phases R₁..Rₙ₋₁ are in place.

### Phase R1 — Cell volumes

**Goal:** compute the k-volume of every k-cube in the complex.

**Algorithm:**
- For a k-cube whose orientation bitmask has active dimensions `{i₁, …, iₖ}`: volume = product of edge lengths along those axes.
- Per-axis case: `vol(k-cube) = ∏_{i ∈ active_dims} axis_length(i)`.
- Per-edge case: requires walking the cube's incident edges and applying the determinant of the Gram matrix of the local edge vectors. For axis-aligned cubical cells with orthogonal edges, this reduces to the product of edge lengths along the active dims (the cross-terms vanish).

**Where it lives:**
- New module: `src/types/cubical_regge_geometry/volumes.rs`.
- Methods on `CubicalReggeGeometry<D>`:
  - `pub fn cell_volume(&self, complex: &LatticeComplex<D>, cell_id: CellId, grade: usize) -> f64`
  - `pub fn top_cell_volume(&self, complex: &LatticeComplex<D>, cell_id: CellId) -> f64` (convenience for the most common case)

**Property tests:**
- Unit grid: every k-cube has k-volume 1.0.
- Per-axis grid: top D-cube volume = `axis_lengths.iter().product()`.
- Per-edge grid with constant lengths: matches per-axis case.

**Effort:** ~120 LOC, ~6 tests. 1 hour.

### Phase R2 — Hinge enumeration + dihedral angles

**Goal:** for each codimension-2 cell (hinge), enumerate the top cubes incident to it and compute the dihedral angle each contributes.

**Algorithm:**
- Hinges in a D-dim cubical complex are (D−2)-cells. Number of hinges = `complex.num_cells(D - 2)`.
- Incident top cubes via `δ ∘ δ`: apply the coboundary operator twice to a hinge to get the top cubes. This is just a sparse matrix product; the result is already cached by `LatticeComplex` after the first call.
- Dihedral angle at a hinge `h` from a top cube `c`: the angle between the two faces of `c` that share `h`. In the per-axis case, this is `arctan2(axis_length[j], axis_length[i])` where `i`, `j` are the two axes of the dihedral's normal plane (the axes inactive in `h` but active in `c`). On the unit-edge / uniform grid, every dihedral angle is exactly π/2.

**Where it lives:**
- New module: `src/types/cubical_regge_geometry/curvature.rs`.
- Helpers in `src/types/lattice_complex/`:
  - `pub fn hinge_top_cube_neighbors(complex: &LatticeComplex<D>, hinge_id: CellId) -> impl Iterator<Item = CellId>` (via the existing coboundary path).
- Methods on `CubicalReggeGeometry<D>`:
  - `pub fn dihedral_angle(&self, complex: &LatticeComplex<D>, top_cube_id: CellId, hinge_id: CellId) -> f64`

**Property tests:**
- Unit grid in any D: every dihedral angle equals π/2.
- Per-axis grid with axes `[a, b]` in 2D: dihedral angle at the vertex from a unit cube is `π/2`, from a stretched cube is `arctan(b/a) + arctan(a/b) = π/2`.
- Symmetry: `dihedral_angle(c, h)` equals the dihedral angle computed from the "other" two faces of `c` at the same hinge.

**Effort:** ~250 LOC, ~10 tests. 3 hours.

### Phase R3 — Deficit angles + Regge action

**Goal:** at every hinge, compute the deficit angle `ε(h) = 2π − Σ θᵢ(h)`, and from that the discrete Einstein-Hilbert action.

**Algorithm:**
- `deficit_angle(hinge_id) = 2π − Σ_{c ∈ incident top cubes} dihedral_angle(c, hinge_id)`. The factor of 2π comes from the normal plane to the hinge being 2-dimensional and full-angle being 2π in flat space.
- Regge action: `S_R = Σ_h volume(h) · deficit_angle(h)`. Volume of a (D−2)-cell from Phase R1.
- Optional Lorentzian variant: when `timelike_axes` flags an axis, dihedral angles in normal planes that include the timelike axis become *imaginary* (rapidities, not Euclidean angles). The Regge action then has a complex phase whose imaginary part encodes the Lorentzian action. Implementation detail: use a `Complex<f64>` accumulator and the rotation-Wick-rotation convention.

**Where it lives:**
- `src/types/cubical_regge_geometry/curvature.rs`:
  - `pub fn deficit_angle(&self, complex: &LatticeComplex<D>, hinge_id: CellId) -> f64`
  - `pub fn regge_action(&self, complex: &LatticeComplex<D>) -> f64`
- For Lorentzian: `pub fn regge_action_lorentzian(&self, complex: &LatticeComplex<D>) -> Complex<f64>`

**Property tests:**
- Unit-edge ℤᴰ lattice (open or periodic): every deficit angle is 0; total Regge action is 0.
- Perturb a single edge length: only the deficit angles of hinges in the affected cube change; total action changes by a computable amount.
- Periodic vs open boundary: action on a torus differs from action on an open grid by exactly the boundary contributions.

**Effort:** ~200 LOC, ~12 tests. 3 hours.

### Phase R4 — Cubical Hodge ⋆ + Laplacian generic over `ChainComplex`

**Goal:** make `manifold/differential/{hodge, laplacian}.rs` work on `Manifold<LatticeComplex<D>, F>`, not just on `Manifold<SimplicialComplex<C>, D>`.

**Algorithm:**
- Cubical Hodge ⋆ on the regular lattice is diagonal: each primal k-cell has a unique dual (D−k)-cell of complementary orientation. The diagonal entry is `volume(dual (D−k)-cell) / volume(primal k-cell)`. On the unit grid both volumes are 1.0 so ⋆ is the identity; with edge-length variation the entries deform.
- Cubical Laplacian on k-forms: `Δ_k = δ_k d_k + d_{k−1} δ_{k−1}`, where `d` is the exterior derivative (via `coboundary_matrix`) and `δ` uses Hodge ⋆.
- This is purely additive on the trait surface — once Hodge ⋆ is implemented for `LatticeComplex<D>`, the existing differential code (which already routes through `ChainComplex::coboundary_matrix` per Stage B) becomes generic over the complex.

**Where it lives:**
- New methods on `CubicalReggeGeometry<D>`:
  - `pub fn hodge_star_matrix(&self, complex: &LatticeComplex<D>, k: usize) -> CsrMatrix<f64>`
- Promote `manifold/differential/{hodge,laplacian}.rs` to `impl<K: ChainComplex, F> Manifold<K, F> where K::Metric: HasHodgeStar` (introduce a small `HasHodgeStar` trait whose two impls are `ReggeGeometry<T>` and `CubicalReggeGeometry<D>`).
  - This is a similar shape to the metric pattern from Stage B option β: an associated capability that some complexes provide and others don't. `CellComplex<C>::Metric = ()` would not provide `HasHodgeStar`, and the Hodge methods would simply be unavailable on `Manifold<CellComplex<_>, _>` — correct behavior.

**Property tests:**
- Unit grid: ⋆_k is the identity matrix for every k.
- Per-axis grid in 2D with axes `[a, b]`: ⋆_0 (vertex → 2-cube) entries are `a·b`; ⋆_1 entries depend on the edge's axis (a/b or b/a). Closed-form test.
- Discrete Hodge decomposition theorem: for any k-form `ω`, `ω = dα + δβ + γ` with `γ` harmonic. Verify on a small lattice.

**Effort:** ~350 LOC (mostly the Hodge ⋆ construction + the small trait), ~15 tests. 5 hours.

**This phase is the keystone.** Once Cubical Hodge ⋆ ships, the existing Laplacian / heat / wave equation paths in `manifold/differential/` work on lattice complexes without further change. The diffusion example we shipped in Stage C (`cubical_heat_diffusion`) currently uses a hand-coded Moore-neighborhood stencil; with Phase R4 in place, that example can be rewritten as a one-line `manifold.laplacian(0)` call.

### Phase R5 — Lorentzian variant + per-cell metric signature

**Goal:** type-safe distinction between Euclidean and Lorentzian cubical geometries, per-cell metric signature computation.

**Algorithm:**
- A `Lorentzian` newtype (or const-generic marker) on `CubicalReggeGeometry<D>` requires the `timelike_axes` field to be `Some(...)` with at least one true entry, and computes per-cell metric tensors with the right signature.
- Per-cell signature is already trivial in the regular-axis case (constant across the lattice, determined by `timelike_axes`). The richer per-edge case requires computing the Gram matrix of the cell's edge vectors and reading the eigenvalue signs.
- Causal-edge constraint: timelike edge lengths squared can be negative in the conventions where `g_{tt} = −1`; the type should allow this and detect light-cone-violating edge assignments.

**Where it lives:**
- Refactor `CubicalReggeGeometry<D>` to take an additional type parameter or expose a marker: `CubicalReggeGeometry<D, S = Euclidean>` with `S ∈ {Euclidean, Lorentzian}`. The Stage C `with_timelike_axes` builder remains the entry point; the type system tracks the choice.
- New: `pub fn metric_tensor_at(&self, complex: &LatticeComplex<D>, cell_id: CellId, grade: usize) -> CausalTensor<f64>` returning the cell's local metric tensor with the right signature.

**Property tests:**
- Euclidean default: every per-cell metric is the D-dim identity (unit grid) or `diag(axis_lengths²)` (per-axis).
- Minkowski (one timelike axis): metric tensor has signature `(−, +, +, +)` in East-Coast convention.
- Light-cone respect: synthesizing an edge length combination that violates the light cone is detected (returns `Err(LightConeViolation)` or similar).

**Effort:** ~250 LOC, ~12 tests. 4 hours.

### Phase R6 — Action gradient + Metropolis updates

**Goal:** compute `∂S_R / ∂(edge_length_i)` for every edge, enabling Monte Carlo Metropolis updates that drive cubical lattice quantum gravity simulations.

**Algorithm:**
- The Regge action is a sum over hinges; an edge `e` enters only via the hinges whose dihedral angles depend on `e`. Hinges with no dependence on `e` contribute zero to `∂S/∂(length_e)`.
- Locality: changing one edge length perturbs only the (constant) finite set of dihedral angles in the cubes containing that edge. The gradient is local — O(2^D) operations per edge.
- Once the gradient is in hand, a Metropolis-style Markov chain over `EdgeLengths { lengths: Vec<f64> }` becomes straightforward; `deep_causality_rand` already ships the RNG infrastructure.

**Where it lives:**
- `src/types/cubical_regge_geometry/dynamics.rs`:
  - `pub fn regge_gradient(&self, complex: &LatticeComplex<D>) -> Vec<f64>` — gradient indexed by edge_id.
  - `pub fn metropolis_update<R: Rng>(&mut self, complex: &LatticeComplex<D>, rng: &mut R, beta: f64) -> AcceptReject`.

**Property tests:**
- Equilibrium: at the unit-edge configuration the gradient is zero (flat space is a stationary point of the Regge action).
- Symmetry: perturbing edge `e_i` and edge `e_j` symmetrically (related by a lattice symmetry) yields equal absolute gradient magnitudes.
- Numerical agreement: gradient via the closed-form computation matches `(S(L + ε·δ_i) − S(L − ε·δ_i)) / (2ε)` to ~5 sig figs.

**Effort:** ~400 LOC, ~10 tests. 6 hours.

**Total Phase R1–R6 effort:** ~1500 LOC, ~65 tests, ~22 hours of focused work. Each phase is independently reviewable.

---

## 4. New capabilities this unlocks

The phases above are not just "completing a math feature." Each one opens a category of computations in the topology crate that is currently impossible or hand-rolled.

### 4.1 Discrete differential geometry on voxel grids

After Phase R4 (cubical Hodge ⋆), the entire `manifold/differential/` API becomes generic over `LatticeComplex<D>`. This is what voxel-based image processing, computational physics on Cartesian grids, computational electromagnetism (FDTD), and discrete-form fluid dynamics need:

- **Heat / diffusion equations** on voxel grids → `manifold.laplacian(0)` instead of hand-coded stencils.
- **Wave equation, Maxwell on cubical lattices** → already partial in `gauge_field_lattice/` for U(1) / SU(N); becomes coupled to the metric (Cubical Regge geometry as the background).
- **Image-processing filters as discrete differential forms** — gradients, divergences, curls of voxel data become typed as morphisms between k-form modules.
- **Persistent homology of voxel data** — the existing Betti number machinery in `ChainComplex` already works on `LatticeComplex<D>`; what's missing is the persistence filtration, which would slot in as a separate change.

### 4.2 Cubical quantum gravity

After Phases R3 (Regge action) + R5 (Lorentzian) + R6 (Metropolis), the crate gains the infrastructure to run cubical-lattice quantum gravity simulations:

- **Path integral over cubical metrics** — Markov chain Monte Carlo over `EdgeLengths { lengths: Vec<f64> }` weighted by `exp(−S_R)` (Euclidean) or `exp(i S_R)` (Lorentzian with Wick rotation).
- **Studying anisotropic spacetimes** — the per-axis structure makes anisotropy first-class; comparing isotropic vs anisotropic vacuum solutions is a natural experiment.
- **Coupling to lattice gauge theory** — `LatticeGaugeField` already lives on `Arc<LatticeComplex<D>>`. With cubical Regge geometry as a per-link metric, the gauge action couples to gravity; backreaction studies and lattice GR-Yang-Mills become possible.
- **Comparison with simplicial CDT** — running both on the same physics question (e.g. effective spectral dimension, area-law for entanglement entropy) tests whether the simplicial result depends on the simplicial topology.

### 4.3 Topological data analysis on grid data

After Phase R4, the existing Hodge-theory machinery (harmonic forms detect holes) works on cubical complexes. Combined with `Neighborhood<LatticeComplex<D>>` strategies:

- **Harmonic forms on voxel grids** — detect holes/voids in 3D scans without first triangulating. Direct application to medical imaging (the `medicine_examples/aneurysm_risk` example currently uses simplicial detection; cubical would skip the triangulation step).
- **Sensor fusion** — the original motivation for issue #487. LIDAR / RF / UWB returns sit naturally on cubical complexes; the topological structure (Betti numbers, harmonic forms) becomes a feature for downstream causality reasoning.

### 4.4 Combination with the HKT machinery — uniform mathematical foundation

This is the structurally interesting part. The crate already provides a HKT layer (`deep_causality_haft`) that lets you write generic code over functors / monads / comonads / adjunctions, with concrete witnesses for each topology kind. The cubical Regge work elevates this from "two backends for the same operations" to **one uniform algebraic spine** spanning topology, geometry, and tensor algebra.

The composition unlocks four patterns:

#### Functor → uniform pointwise transforms

`Manifold<K, F>` with `Functor<K>` is already in place. After cubical Regge ships, the *same* `fmap` call transforms a field on a simplicial mesh, a voxel grid, or a user-defined cell complex — with the right metric-aware semantics derived from `K::Metric`. Image-processing pipelines (filter chains on `Manifold<LatticeComplex<D>, f64>`) compose with simulation pipelines (filter chains on `Manifold<SimplicialComplex<C>, FloatType>`) under one type. Cross-domain pipelines that ingest voxel sensor data, transform it via differential operators, and feed the result into a simplicial mesh of the same underlying region become a fmap composition rather than ad-hoc wiring.

#### CoMonad → uniform stencil operations

`CoMonad::extend` on `SimplicialManifold` already does cellular-automaton-style spatial computations; `GenericManifoldWitness<K>` (shipped in Stage C) plus the `Neighborhood<K>` strategies extend this to any `ChainComplex`. After Phase R4 ships, the `extend` closure can call `manifold.laplacian(grade)` or `manifold.exterior_derivative(grade)` instead of hand-rolled stencils — and the compiler picks the right backend (simplicial Cayley-Menger vs. cubical Gram-determinant) based on the complex type. **GNN-style architectures, cellular automata, and discrete PDE solvers are then one extend call apart from each other**, differing only by the kernel passed to `extend`. The example we shipped (`cubical_heat_diffusion`) becomes a one-liner.

#### Adjunction → uniform discretization ↔ continuum bridge

The `BoundaryWitness` / `StokesAdjunction` machinery in `extensions::hkt_gauge::hkt_adjunction_stokes` already abstracts discretize-realize for differential forms. With cubical Regge geometry in place, the same adjunction works for voxel-discretized continuum fields (e.g. CT-scan reconstruction of a Riemannian field) as it does for simplicial-meshed ones. The mathematical content of Stokes' theorem — `∫_∂c ω = ∫_c dω` — becomes a single law parameterized by the underlying complex.

#### Tensor algebra ↔ topology composition

`deep_causality_tensor::CausalTensor<T>` is the field-data type carried by `Manifold<K, F>`. Discrete differential forms in the crate are k-form tensors over a complex's k-skeleton. After cubical Regge ships:

- A **k-form on a simplicial complex** and a **k-form on a cubical complex** are both `CausalTensor<F>` indexed by `K::num_cells(k)`.
- `tensor.fmap` (pointwise) commutes with `manifold.exterior_derivative` (algebraic structure).
- This means you can write algorithms in `CausalTensor` operations and have them automatically respect the differential structure of whatever complex they live on.
- Combined with `deep_causality_metric::Metric` (the metric signature types), Lorentzian / Euclidean / general Clifford-algebra computations sit on the same tensor backbone with the right signs derived from `K::Metric`.

The net architectural payoff is: **the topology crate becomes a discrete differential geometry library where the complex type, the metric type, the tensor type, and the HKT structure are all coordinated through associated types — a uniform mathematical foundation for problems that currently require five different libraries.**

### 4.5 Connections to other DeepCausality crates

Each unlocked capability has a downstream consumer in the project's existing crate graph:

- **`deep_causality_physics`** (lattice gauge theory, GR, electroweak, etc.) — cubical Regge gives it a metric backbone for the lattice. Backreaction studies, Einstein-Yang-Mills, anisotropic spacetimes.
- **`deep_causality_effects`** (effect types over heterogeneous graphs) — discrete-form evolutions on cubical complexes become effect kinds. Cellular-automaton-style effect propagation through voxel grids fits naturally.
- **`deep_causality_ethos`** (programmable ethics / verification rules) — light-cone enforcement at the type level (Phase R5) is itself a verification capability. A "no closed timelike curves" Ethos rule becomes a type-level constraint.
- **`deep_causality_uncertain`** (uncertain first-order types) — edge lengths can carry uncertainty; the Regge action propagates uncertainty through the geometry, giving discrete uncertain gravity.
- **`deep_causality_topology` itself** — the `topology` capability spec gets a clean parametrization-by-complex story; future capabilities (persistence, sheaves, Khovanov homology, etc.) slot in along the same axis.

---

## 5. What this note doesn't address (out of scope)

Listed for completeness so the next contributor knows the boundaries:

- **Sparse cubical complexes** — only-active-cubes representation. Required for high-resolution voxel grids (the original sensor-fusion motivation at scale). Independent of Regge geometry; proposal already lists it as a deferred follow-up to `add-cubical-complexes`.
- **GPU paths** — the existing `_cpu` / `_impl` naming pattern preserves the option of GPU backends. Each Phase R method could have a GPU variant. Out of scope for the math; in scope for a separate perf-track change set.
- **Non-cubical regular tilings** (hex, kagome, triangular) — already present as `HoneycombLattice`, `KagomeLattice`, etc. They're related to cubical Regge geometry by a category of regular tilings, but each needs its own Regge analog (e.g., hex Regge calculus). Not part of the cubical line.
- **Persistent homology, sheaves, Khovanov-style invariants** — orthogonal extensions to `ChainComplex`. Each is its own change set.
- **Categorical coherence proofs** — the HKT machinery's structural laws (functoriality, monad laws, adjunction unit/counit equations) for `LatticeComplex<D>` need verification once the metric-dependent operations land. Adding property-based tests via `proptest` would be the natural way.

---

## 6. Suggested change-set naming

When this work is opened as a follow-up, suggested OpenSpec change name: **`add-cubical-regge-calculus`**. Phases R1–R3 could be one change set (geometric core: volumes, hinges, action). Phases R4–R6 could be a second (analytical core: Hodge, Lorentzian, dynamics). Each is independently testable and reviewable.

The same stage-gate protocol used in `add-cubical-complexes` (per-stage sign-off + commit, no agent commits) is the recommended workflow.

---

## 7. Direction after Cubical Regge calculus: Hodge decomposition

Once Phases R1–R6 have shipped, the question is what to build *with* the cubical Regge geometric layer that produces near-term scientific and engineering value without entering the contested territory of lattice quantum gravity. The recommended target is the **discrete Hodge–Helmholtz decomposition as a uniform discrete-form operation across complex types**. It is a sharply-scoped, well-settled-mathematics contribution that exercises the full HKT machinery and produces immediately applicable tools.

### 7.1 Why this comes next, and not lattice quantum gravity

Lattice quantum gravity (cubical CDT-style comparison) is genuinely interesting and the cubical Regge work would be one of the prerequisites for it. But:

- The science is **not settled** (the CDT spectral-dimension result is empirical, the universality is unknown, and the cubical reformulation is an open research question without published precedent at production scale).
- A credible CDT-vs-cubical comparison is a **6–12 month research program**, not a library deliverable.
- The required additional infrastructure (Markov-chain Monte Carlo over edge-length configurations, ergodic Pachner moves on the simplicial side, CDT's foliation-aware `SimplicialComplex` variant) is itself ~100–150 hours of further work *before* any physics is touched.

The Hodge decomposition direction is the opposite of this on every axis: settled science, validated reference implementations exist for the simplicial path, the cubical Hodge ⋆ on per-edge metrics is a small (~1 week of derivation) but real gap in the published literature that we can close cleanly, and the downstream applications are concrete and immediate.

### 7.2 What it is

The Hodge–Helmholtz decomposition states that any k-form `ω` on a closed complex admits a unique orthogonal split

```
ω  =  dα   +   δβ   +   γ
       │       │       │
       │       │       └─ harmonic part (closed and co-closed; carries topology)
       │       └───────── co-exact part (divergence-free / "curl-like")
       └───────────────── exact part (curl-free / "gradient-like")
```

For smooth manifolds this is Hodge's 1941 theorem. The discrete version on simplicial complexes is from Hirani's 2003 Caltech thesis (Discrete Exterior Calculus). The cubical version is folkloric — scattered through DEC and lattice-physics papers, with no canonical reference implementation in a typed library.

Mathematically it reduces to a sparse least-squares problem: solve `Δα = δω` for the exact part, `Δβ = dω` for the co-exact part, residual is harmonic. The `Δ` is the Hodge–Laplacian — `δd + dδ` — which is exactly what Phase R4 of this note's roadmap delivers for cubical complexes.

### 7.3 The categorical-uniformity contribution

The algorithm itself is settled. What's missing in the existing ecosystem is **uniform expression across complex types with static-dispatch type safety**:

| Library | Hodge on simplicial? | Hodge on cubical? | Uniform API across both? |
|---|---|---|---|
| PyDEC (Bell, Hirani; Python) | ✔ | ✘ | ✘ |
| Discrete-form modules in FEniCS / Firedrake (C++ / Python) | ✔ (FEM-flavor) | ✘ | ✘ |
| Voxel-based discrete operators (SciPy `divergence` / `curl`, numpy.gradient) | ✘ | partial (no Hodge ⋆) | ✘ |
| Catlab.jl (Julia) | conceptually ✔ | conceptually ✔ | yes, but dynamically dispatched |
| **DeepCausality after this work** | ✔ | ✔ | **✔, zero-cost static dispatch via `ChainComplex`** |

The HKT machinery in `deep_causality_haft` is what makes the "one implementation, multiple backends" property load-bearing rather than decorative. The user-facing function

```rust
fn hodge_decompose<K, F>(manifold: &Manifold<K, F>, k: usize)
    -> HodgeDecomposition<F>
where K: ChainComplex,
      K::Metric: HasHodgeStar,
```

works on simplicial meshes, voxel grids, and user-defined cell complexes with no code change at the call site. The monomorphizer picks the right Hodge ⋆ implementation (Cayley-Menger-derived for simplicial, dual/primal volume ratio for cubical) at compile time. No `dyn`, no enum dispatch, no per-backend rewriting.

### 7.4 Phase plan (~90 hours focused work)

Smaller than the Cubical Regge phases R1–R6 because most of the heavy lifting is already in this note's roadmap. Phases:

| Phase | Description | Effort |
|---|---|---|
| **H1** | `HasHodgeStar` capability trait. Implementors: `ReggeGeometry<T>` (simplicial, exists), `CubicalReggeGeometry<D>` (delivered by Phase R4 of this note). Trait gates the Hodge-dependent operators. | ~5 h |
| **H2** | `HodgeDecomposition<F>` data structure + the decomposition routine. Uniform over `K: ChainComplex` where `K::Metric: HasHodgeStar`. Sparse least-squares via existing `deep_causality_sparse` infrastructure. Returns `(exact, co_exact, harmonic)` triple as three `CausalTensor<F>` instances. | ~25 h |
| **H3** | Two-backend property test suite. Orthogonality, exactness/co-exactness, harmonic-part-dimension-equals-Betti-number, cross-backend agreement on sampled continuous fields. | ~10 h |
| **H4** | Per-edge cubical Hodge ⋆ derivation and implementation. *The only piece with mathematical novelty.* Derive the closed form for the diagonal entries of ⋆_k on a cubical complex with arbitrary per-edge lengths, using the dual/primal volume ratio. Cross-check against simplicial result on a complex that admits both decompositions (e.g. unit square with both diagonal triangulation and trivial cubical decomposition). | ~15 h |
| **H5** | Validation against published references. Reproduce Hirani 2003 thesis examples (simplicial). Analytical test fields on `LatticeComplex<2>` with known decompositions (vortex + gradient + harmonic on a torus). Cross-validate against PyDEC on the simplicial path. | ~15 h |
| **H6** | Example: vector-field denoising on a 2D/3D field. Decomposes noisy data into structured + harmonic + co-exact components; harmonic and high-frequency-co-exact components are discarded for denoising. Visualizable. Demonstrates the practical payoff. | ~10 h |
| **H7** | Methods note / writeup. Section in `docs/`, blog post draft, or OpenSpec note documenting the categorical-uniformity argument with code samples. | ~10 h |

**Total: ~90 hours focused work, ~5–6 weeks at sustained pace.**

H4 is the only phase with research content (closing a small gap in the published per-edge cubical Hodge ⋆ literature); the other six are engineering on well-settled foundations.

### 7.5 Validation strategy

Three nested validations, ordered from cheapest to costliest:

1. **Per-backend mathematical correctness.** Orthogonality (`⟨dα, δβ⟩ = 0`), exactness (`δ(dα) = 0`), co-exactness (`d(δβ) = 0`), harmonic-dimension equals `Betti_k`. These are property tests; they pass or fail on a per-complex basis.
2. **Cross-backend agreement.** A continuous test field `f(x, y)` sampled on (a) a triangulated mesh of `[0,1]²` and (b) a voxel grid of the same square. Decompose each. Assert the recovered exact/co-exact/harmonic parts agree at common sample points up to discretization error. Show convergence: max error → 0 as mesh resolution → ∞ on both backends.
3. **Reference-implementation parity.** Run PyDEC on the simplicial path on a fixed test field. Run DeepCausality on the same field. Assert the decompositions match up to numerical tolerance. PyDEC is the de-facto simplicial reference; matching it on simplicial *and* delivering working cubical is the methods-paper-grade claim.

### 7.6 Reference points (all settled science)

- **Hirani, A. N.** "Discrete Exterior Calculus." PhD thesis, Caltech, 2003. Canonical reference for simplicial DEC; covers Hodge decomposition in full implementational detail.
- **Desbrun, Kanso, Tong.** "Discrete Differential Forms for Computational Modeling." SIGGRAPH 2005 course notes. Most accessible introduction; algorithm-level pseudocode.
- **Crane, K.** "Discrete Differential Geometry: An Applied Introduction." CMU course notes (regularly updated, freely available). Best applied treatment with worked examples and code.
- **Tong, Y., Lombeyda, S., Hirani, A. N., Desbrun, M.** "Discrete Multiscale Vector Field Decomposition." ACM TOG 22(3), 2003. The foundational sensor-fusion / vector field denoising paper using Hodge decomposition.
- **Bhatia, H., Norgard, G., Pascucci, V., Bremer, P.-T.** "The Helmholtz-Hodge Decomposition — A Survey." IEEE TVCG 19(8), 2013. Comprehensive applications survey: fluid simulation, computer graphics, sensor data, medical imaging.
- **Brandt, C., Scandolo, L., Eisemann, E., Hildebrandt, K.** "Spectral processing of tangential vector fields." Computer Graphics Forum 36(6), 2017. Modern spectral / Hodge-based vector field analysis.

### 7.7 What this unlocks

Once `hodge_decompose` is shipped, the Laplacian eigenbasis comes nearly for free (harmonic forms are the kernel of Δ; spectral methods are a one-line extension). That in turn unlocks:

- **Vector-field denoising** — separate signal from noise via harmonic-part filtering. Direct application to the sensor-fusion motivation of issue #487.
- **Topological-feature detection** — Betti numbers via harmonic-basis dimension; an alternative to persistent homology that's faster for specific feature classes.
- **Conservative discretization for fluid simulation** — incompressibility enforced by projecting out the exact part.
- **Vector field design (computer graphics)** — editable, structure-preserving vector fields on surfaces.
- **Shape descriptors** — Heat Kernel Signature (HKS), Wave Kernel Signature (WKS) build directly on the Hodge–Laplacian spectrum. These are the standard machinery for non-rigid shape matching.
- **Coupling with `deep_causality_effects`** — discrete-form decompositions become effect kinds in the Effect Ethos framework. A "field must be solenoidal" constraint becomes a typed verification rule.

None of these are new science. All are immediately applicable, validatable against existing tools, and unlocked uniformly across complex types — which no existing library delivers.

### 7.8 Honest caveats

1. **The categorical-uniformity claim has to be load-bearing in the API, not decorative.** The library will be judged on whether `hodge_decompose<K, F>` actually works at the call site with zero per-backend conditional logic. The right success test: a single user-facing example that decomposes the same physical field on a simplicial mesh and on a voxel grid, with the call-site code byte-identical except for the manifold construction.
2. **PyDEC is the simplicial standard for this work.** Any methods paper or write-up will be compared to it directly. DeepCausality must at minimum match PyDEC's correctness on the simplicial path. The selling point is *cubical also works without rewriting*, not *simplicial works better than PyDEC*.
3. **The per-edge cubical Hodge ⋆ derivation (H4) is the one piece of genuine math work.** On unit grids cubical Hodge ⋆ is trivial (identity). On per-axis grids it's standard (diagonal of axis-length ratios). On per-edge metrics with general edge lengths, the diagonal entries follow from dual/primal volume ratios using the cubical Regge geometry framework, but the worked-out closed-form expression I am not confident exists in published form. Likely a ~1-week derivation effort with cross-checks against the simplicial case where both decompositions are valid (e.g. a square that's both triangulated and seen as a 2-cube). This is the real research content of the work, small but genuine.
4. **The application demos benefit from a real dataset.** Synthetic test fields suffice for the methods paper, but a downstream paper applying the decomposition to real sensor data (e.g. KITTI for autonomous driving, BraTS for medical imaging, a published flow-field dataset) makes the case far stronger. Budgeting one demo on public data is reasonable; making the entire deliverable contingent on it is not.

### 7.9 Suggested change-set name and sequencing

When this work is opened, suggested OpenSpec change name: **`add-hodge-decomposition`**.

Sequencing relative to the rest of the Cubical Regge roadmap:

1. **`add-cubical-regge-calculus` (R1–R3)** — geometric core: cell volumes, hinges, deficit angles, Regge action.
2. **`add-cubical-regge-calculus` (R4–R6)** — analytical core: cubical Hodge ⋆ (making `manifold/differential/{hodge,laplacian}` generic), Lorentzian variant, action gradient. *This is the prerequisite for the Hodge decomposition work.*
3. **`add-hodge-decomposition`** — the targeted, settled-science contribution described in this section. Builds on R4. Does not require R5 (Lorentzian) or R6 (dynamics).

Steps 1–3 in sequence: ~22 + (R4–R6 ~20) + 90 ≈ **~130 hours of focused work** total. Doable as a sustained 2–3 month effort by one developer, or split across multiple change sets with explicit stage gates per the protocol used in `add-cubical-complexes`.

## 8. Direction after Hodge decomposition: causal-graph analysis of turbulent flows

The Hodge decomposition is a natural foundation for *discrete fluid analysis*. Once it ships (§7), a sharply-scoped follow-up direction opens: applying DeepCausality's causal-reasoning machinery (`Causaloid`, `Context`, `Effect Ethos`) to turbulent flow data, producing a class of analyses no existing fluid-dynamics toolkit offers. This section scopes that direction with the same honesty as §7 — what is tractable, what is not, where the real contribution lives, and what should be left alone.

### 8.1 Why fluid dynamics is a natural target, and where to be careful

The discrete Hodge–Helmholtz decomposition is *the* foundational tool for structure-preserving discrete fluid simulation. Incompressible Navier–Stokes is essentially:

- Velocity field `u` satisfies `∇·u = 0` (incompressibility).
- The Hodge decomposition produces exactly this projection — "Helmholtz projection" in CFD textbooks.
- The three orthogonal pieces — exact, co-exact, harmonic — map cleanly onto **gradient flow** (laminar inflow/outflow), **vortical flow** (where turbulent kinetic energy lives), and **large-scale circulation** (topology-defined).

The downstream payoff after §7 is correspondingly large *for the right class of problems*. But turbulence is the unsolved problem of classical physics; it is easy to overpromise here. Strict scoping discipline applies.

### 8.2 What's tractable, what's not

**Tractable with the toolkit on the §7 foundation:**

| Capability | How |
|---|---|
| Incompressibility enforcement | Project velocity onto the co-exact (divergence-free) component each timestep — standard Chorin projection method |
| Vorticity-based formulations | Vorticity = `d(velocity)`; evolved via vorticity transport; streamfunction recovered through the Hodge inverse |
| Structure-preserving discretization | Discrete differential operators preserve `∂² = 0` exactly, which preserves Kelvin's circulation theorem at the discrete level — important for long-time turbulence simulation |
| Flow-feature analysis from measured or DNS data | Decompose any vector field into laminar + rotational + harmonic components; locate coherent vortices via the co-exact part; compute helicity, enstrophy, spectral energy distribution |
| Idealized turbulence test problems | 2D vortex dynamics, vortex shedding around obstacles, Taylor–Green vortex decay, shear-layer instabilities — the standard demos in DEC-fluid papers |
| Multiscale analysis | The Laplacian eigenbasis (nearly free once Hodge ships) gives a spectral decomposition of the flow — generalized Fourier modes on arbitrary complexes |

**Not tractable, and the toolkit should not pretend otherwise:**

- **High-Reynolds-number direct numerical simulation (DNS).** Production DNS uses 10⁹–10¹² grid points on dedicated HPC clusters for weeks at a time. DeepCausality is not an HPC framework. Don't compete here.
- **Production engineering CFD.** OpenFOAM, ANSYS Fluent, etc. have decades of mesh adaptation, turbulence closure models (k-ε, k-ω SST, RSM, dynamic-LES), boundary-layer treatments, multiphase flow, etc. Don't compete here either.
- **The turbulence closure problem itself.** Modeling the sub-grid-scale energy cascade in a way that doesn't require resolving all scales is an open problem in physics. The toolkit doesn't solve it.

### 8.3 Three interpretations of "causal structure in fluid dynamics"

The phrase has at least three distinct meanings, each pointing at different work:

**(a) Causal flow networks — information-theoretic causality between flow events.**

Active research direction. Lozano-Durán et al. (Stanford / Caltech), Madhusudanan et al., and others use transfer entropy, Granger causality, SHAP-style attribution, and other information-theoretic measures to identify *what flow events cause what other flow events* in turbulent boundary layers, shear flows, and channel flows. The output is a directed causal graph over flow events identified at coarse scales (e.g. coherent structures, ejection / sweep events near walls).

DeepCausality is *uniquely positioned* for this interpretation. Its `Causaloid`, `Context`, and `Effect Ethos` machinery exists precisely to model causal-event reasoning. Coupling the discrete-form analysis (§7) with the causal-graph machinery produces a workflow no existing fluid-dynamics tool offers. **This is the recommended target.**

**(b) Light-cone / hyperbolic causal structure — for compressible and relativistic flow.**

Sound waves, shocks, relativistic fluids (GRMHD, accretion disks, neutron-star interiors) have a finite signal-propagation speed creating a causal cone at each spacetime point — directly analogous to light cones in relativity, with sound speed in place of `c`. Phase R5 of this note (Lorentzian cubical Regge) gives this structure: `timelike_axes` defines spacelike vs timelike; the local per-cell metric defines the local sound cone.

`deep_causality_physics::mhd` and `grmhd` modules already exist in the crate. Coupling them to cubical Regge geometry gives **structure-preserving GRMHD on a cubical lattice** — a real research direction in numerical relativity and computational astrophysics. But the production codes here (Einstein Toolkit, GRHydro) are HPC-scale and well-developed; the DeepCausality angle competes only on structure preservation and typed correctness, not on raw simulation throughput.

**(c) DeepCausality's general causal-reasoning framework applied to fluid systems.**

The crate's `Causaloid` / `Context` / `Effect Ethos` infrastructure can express:

- **Conservation laws as Ethos verification rules** — "mass is conserved", "circulation is preserved on closed contours" become typed constraints that the system enforces.
- **Local update rules as `CoMonad::extend`** — time evolution is comonadic, with a Navier–Stokes kernel instead of a heat kernel.
- **Causal propagation as light-cone constraints** — no information moves faster than the local sound speed; the type system can enforce this when the metric is Lorentzian.

This isn't a *fluid solver* — it's a *causal-reasoning layer over fluid solvers*. Useful for: trusted simulation (knowing the conservation laws are respected because the type system enforces them), counterfactual analysis ("what if this vortex hadn't formed?"), and verifiable physics pipelines for safety-critical applications.

### 8.4 The targeted contribution: causal-graph analysis of flow data

Of the three interpretations, **(a) is the right target.** Reasons:

- It leverages the *unique* parts of DeepCausality (causal-graph reasoning, Effect Ethos) — not just the topology layer.
- It builds on settled foundational tools (Hodge decomposition from §7, information-theoretic causality measures from a settled literature).
- It produces a *publishable* methods-paper contribution: "causal-graph-aware turbulence analysis using discrete-form decompositions and typed causal-reasoning."
- It does not compete with production CFD or HPC codes — it consumes their outputs.
- The combination of (i) typed discrete-form library + (ii) causal-graph machinery + (iii) Ethos verification is something **no existing fluid-dynamics tool offers.**

The pipeline:

1. **Ingest flow data** — DNS snapshots (JHU Turbulence Database, Johns Hopkins Turbulence Database has freely available data), PIV measurements, weather/ocean fields, or your own small-scale simulations from §8.2's tractable list.
2. **Extract structured flow features via Hodge decomposition** — identify coherent vortices (co-exact part), shear regions, dissipation events, large-scale circulation patterns (harmonic part).
3. **Build a `Context` over the flow features** — each detected feature becomes a context node; spatial and temporal proximity define context relations.
4. **Construct `Causaloid` chains linking flow events** — using transfer entropy, Granger causality, or local-flow-perturbation analysis to identify which events causally drive which downstream events.
5. **Verify physical constraints as Ethos rules** — conservation of mass, momentum, energy, helicity, and (for compressible flow) the local sound-cone constraint as typed verification rules.
6. **Output: a verified causal graph over the flow** — annotated with conservation-law compliance, ready for downstream analysis (counterfactual reasoning, predictive modeling, anomaly detection).

### 8.5 Phase plan

Smaller than the Cubical Regge phases R1–R6 and the Hodge decomposition phases H1–H7 because most of the heavy lifting is already in §7's roadmap and in the existing DeepCausality crates. Phases:

| Phase | Description | Effort |
|---|---|---|
| **F1** | Flow-data ingestion adapters. Read DNS snapshots (HDF5 / NetCDF for JHU Turbulence Database format; OpenFOAM `.vtk` for openfoam-style data). Map onto `Manifold<LatticeComplex<D>, F>` or `Manifold<SimplicialComplex<C>, F>` depending on grid type. | ~30 h |
| **F2** | Coherent-structure extraction via Hodge decomposition. Each detected vortex / shear / sink becomes a typed feature carrying its spatial extent and time of detection. | ~25 h |
| **F3** | `Context` builder over flow features. Spatial-proximity edges via the existing `Neighborhood<K>` strategies. Temporal-proximity edges via a windowing strategy over the time series. | ~20 h |
| **F4** | `Causaloid` chain construction. Implement the information-theoretic causality measures: transfer entropy (Schreiber 2000) as the primary measure, with Granger causality as a fallback for time series with known linear structure. Output is a directed graph over flow features weighted by causal influence. | ~50 h |
| **F5** | Effect Ethos rules for fluid physics. Mass conservation, momentum conservation, energy conservation, helicity preservation (for inviscid flow), sound-cone respect (for compressible flow). Each becomes a typed verification rule attached to the simulation or analysis. | ~30 h |
| **F6** | End-to-end demo on a public dataset. Recommended: a JHU Turbulence Database snapshot of forced isotropic turbulence or channel flow. Walk through the full pipeline: ingest → extract → context → causaloid → ethos verification → visualize. | ~30 h |
| **F7** | Methods note / writeup. Document the typed pipeline. Discuss what's novel (typed causal-graph reasoning over discrete-form decompositions) and what isn't (the underlying information-theoretic measures are settled). | ~15 h |

**Total: ~200 hours focused work — ~2–3 months at sustained pace.**

F4 is the phase with the most depth (transfer entropy on multivariate flow time series is a serious topic with its own active literature). The other six phases are well-established adapter / interface work.

### 8.6 Validation strategy

Three nested validations:

1. **Synthetic ground-truth.** Generate a small flow where the causal structure is *known by construction* (e.g. a forced 2D Kolmogorov flow with a known forcing → response chain). Apply the full pipeline; verify the output causal graph recovers the known structure.

2. **Reproduce a published result.** Take a published causal-flow-network paper (Lozano-Durán et al. or similar) that uses transfer entropy on DNS data and reproduce one of their causal-graph results on the same dataset. Quantitative agreement with the literature is the strongest validation.

3. **Conservation-law compliance.** On a DNS snapshot, verify the Hodge decomposition + Ethos rules correctly flag conservation violations introduced by numerical roundoff or coarse-graining. The Ethos rules should *not* flag clean DNS data; should flag intentionally perturbed data.

### 8.7 Reference points

- **Schreiber, T.** "Measuring information transfer." *Phys. Rev. Lett.* 85, 461 (2000). The original transfer-entropy paper. Used everywhere in causal-network analysis.
- **Lozano-Durán, A., Bae, H. J.** "Self-similar hierarchical organization of momentum transfer in wall-bounded turbulence." *Phys. Rev. Lett.* 126, 134503 (2021). Recent example of causal-network analysis applied to turbulent boundary layers.
- **Lozano-Durán, A., Bae, H. J., Encinar, M. P.** "Causality of energy-containing eddies in wall turbulence." *Journal of Fluid Mechanics* 882, A2 (2020). Foundational paper for the causal-network-in-turbulence literature.
- **Mullen, P., McNamara, A., Tong, Y., Desbrun, M.** "Energy-preserving integrators for fluid animation." *ACM TOG* 28(3), 2009. DEC-based fluid solver; structure preservation in action.
- **Pavlov, D., Mullen, P., Tong, Y., Kanso, E., Marsden, J., Desbrun, M.** "Structure-preserving discretization of incompressible fluids." *Physica D* 240, 2011. The reference paper for structure-preserving discrete fluid solvers.
- **Bhatia, H. et al.** "The Helmholtz-Hodge Decomposition — A Survey." *IEEE TVCG* 19(8), 2013. Applications survey explicitly covering fluid analysis.
- **JHU Turbulence Database.** http://turbulence.pha.jhu.edu/ — freely available DNS data for isotropic turbulence, channel flow, MHD turbulence, etc. The standard public benchmark for turbulence-analysis methods.

### 8.8 What this unlocks

After F1–F7 ship:

- **Causal-graph-aware turbulence analysis** — identifying which flow events drive which downstream events in DNS or measurement data, with typed conservation-law verification.
- **Counterfactual analysis of flow data** — "what if this vortex hadn't formed?" — by manipulating the causal graph and propagating consequences.
- **Anomaly detection in flow time series** — Ethos rules flag conservation violations; causal-graph changes flag structural anomalies (e.g. a vortex appearing without a precursor cause).
- **Safety-critical sensor-fusion** for fluid-borne sensor data — atmospheric measurements, ocean-current sensing, blood-flow analysis in medical imaging. The Ethos layer turns physical constraints into mechanical correctness checks.
- **Coupling with `deep_causality_uncertain`** — sensor measurements carry uncertainty; the causal-graph machinery propagates uncertainty through the inferred causal structure.
- **A foundation for `deep_causality_ethos` rule libraries** — fluid-physics conservation laws become a reusable Ethos rule set that downstream applications inherit.

None of these are new in their individual ingredients. The combination — typed causal-graph reasoning over discrete-form decompositions with first-class verification — is the contribution.

### 8.9 Honest caveats

1. **The toolkit is for *analysis*, not for *simulation* of high-Reynolds turbulence.** Consume DNS data from elsewhere; produce causal graphs and verified analyses. Don't try to be a competitive flow solver.
2. **The information-theoretic causality measures (transfer entropy etc.) are themselves contested in the turbulence community.** They identify *statistical association* with directed temporal precedence, not Newtonian cause-and-effect in any rigorous sense. The toolkit should be honest about this — output causal graphs are statistical, not mechanistic. Phase F4's documentation must say so explicitly.
3. **The conservation-law Ethos rules must be derived carefully.** Discrete Hodge decomposition preserves some conservation laws exactly (curl-of-grad = 0, div-of-curl = 0) but not others (kinetic energy is preserved by the right time-integrator but easy to lose with a careless one). Each Ethos rule needs a citation to the discretization scheme that makes it exact, or an explicit tolerance for schemes that make it approximate.
4. **Public DNS data has specific licenses.** The JHU Turbulence Database is freely accessible for research but downstream redistribution has conditions. Any demo using public DNS data needs the right attribution and license notes.
5. **The "novel contribution" framing depends on the existing causal-flow-network literature being narrower than DeepCausality's framework.** It is — those papers use ad-hoc Python pipelines, not typed causal-reasoning libraries — but the claim deserves due-diligence-level literature review before publication.

### 8.10 Suggested change-set name and sequencing

When this work is opened, suggested OpenSpec change name: **`add-causal-flow-analysis`**.

Sequencing relative to the rest of the roadmap:

1. **`add-cubical-regge-calculus` (R1–R3)** — geometric core.
2. **`add-cubical-regge-calculus` (R4–R6)** — analytical core (Hodge ⋆, Lorentzian, dynamics).
3. **`add-hodge-decomposition`** — uniform Hodge decomposition (§7).
4. **`add-causal-flow-analysis`** — causal-graph analysis of flows (this section). Builds on §7; consumes the discrete-form layer; layers DeepCausality's causal-reasoning machinery on top.

Steps 1–4 in sequence: ~22 + ~20 + ~90 + ~200 ≈ **~330 hours of focused work** total. Realistic as a sustained 6–9 month roadmap by one developer, or split across multiple change sets with explicit stage gates per the protocol used in `add-cubical-complexes`.

The cumulative outcome: a Rust library that delivers structure-preserving discrete differential geometry, uniform Hodge decomposition, *and* typed causal-graph reasoning over fluid-flow data — a combination that doesn't exist elsewhere and that lands DeepCausality squarely in a small set of high-value applied research niches (turbulence analysis, safety-critical sensor fusion, verifiable physics pipelines).

## 9. Bottom line

Stage C of `add-cubical-complexes` shipped the *scaffolding* — type-correct edge-length storage with all four uniformity levels and Lorentzian axis flags. The full Cubical Regge calculus is ~6 phases of focused additive work, totaling ~22 hours / ~1500 LOC. None of those phases require changes to the trait surface (`ChainComplex`, `Neighborhood`, `Manifold<K, F>`) shipped in Stages A–C; everything slots in via new methods on `CubicalReggeGeometry<D>` and one small additional trait (`HasHodgeStar`) gating the Hodge-dependent differential operators.

The payoff is a topology crate where the algebraic spine (functor / monad / comonad / adjunction over discrete forms on chain complexes) covers simplicial *and* cubical paths under one uniform mathematical foundation — and where the existing HKT machinery, tensor algebra, metric-signature types, and gauge-field types all compose with that spine without bespoke per-backend wiring. For sensor-fusion workloads (the original motivation of #487), for lattice quantum gravity research, for discrete-form numerical PDE methods, and for grid-native topological data analysis, this is the foundation those applications would build on.
