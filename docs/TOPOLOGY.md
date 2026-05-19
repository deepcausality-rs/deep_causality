# DeepCausality Topology: Geometric Foundations

`deep_causality_topology` provides the discrete geometric structures required to model causal systems in space and time. It focuses on **Simplicial Homology** and **Discrete Differential Geometry**.

---

## 🏗️ Core Concept: Simplicial Complex

The foundational unit is the `SimplicialComplex`. Unlike a simple graph (which only has vertices and edges), a simplicial complex handles higher-dimensional "volumes":

*   **0-simplex**: Vertex (Point)
*   **1-simplex**: Edge (Line)
*   **2-simplex**: Face (Triangle)
*   **3-simplex**: Volume (Tetrahedron)
*   **k-simplex**: k-dimensional generalization

These are organized into **Skeletons**, where the k-th skeleton contains all k-simplices.

### The Operators
The crate explicitly computes and stores three critical topological operators:
1.  **Boundary Operator (∂)**: Maps a (k)-volume to its (k-1)-boundary (e.g., triangle -> 3 edges).
2.  **Coboundary Operator (δ)**: The dual of boundary. Maps a (k)-simplex to the (k+1)-simplices it is part of.
3.  **Hodge Star (⋆)**: Maps a k-form to an (N-k)-form, enabling duality between geometry and fields.

---

## 🌐 The Manifold

A `Manifold<T>` is a "safe" wrapper around a `SimplicialComplex` that enforces geometric guarantees required for physics:
1.  **Orientation**: The manifold must have a consistent "up/down" or "in/out" direction.
2.  **Link Condition**: Ensures the local neighborhood of every point looks like a disk/ball (no "pinched" points).

### Data on the Manifold
While the `SimplicialComplex` stores the shape, the `Manifold` stores the **Field Data** (`CausalTensor<T>`) living on that shape.
This represents physical quantities (like Temperature, Electric Field, Probability Amplitude) distributed over space.

---

## 🧮 Regge Geometry (Discrete Gravity)

The crate includes `ReggeGeometry` to model curved spacetime without continuous manifolds.
*   It uses **Edge Lengths** to define the geometry metric.
*   Curvature is calculated via **Deficit Angles** around bones (hinges).
*   This allows simulating gravity and relativistic effects on a discrete mesh.

---

## 🔗 Topology as Comonad

DeepCausality treats Topology as a **Comonad**.
*   **Monad**: "Into the future" (Sequencing effects).
*   **Comonad**: "Into the neighborhood" (Contextual extraction).

In this crate, `extend` and `extract` allow a cell to update its state based on its neighbors (e.g., Heat Diffusion, Cellular Automata), leveraging the `coboundary` operator for efficient adjacency lookups.

---

## 🟦 LatticeComplex / Cubical Complexes

Where a simplicial complex tessellates space with triangles / tetrahedra, a **cubical complex** tessellates with hypercubes — line segments in 1D, squares in 2D, cubes in 3D, hypercubes in higher dimensions. For grid-native data (voxel grids, regular sensor arrays, image stacks, LIDAR / RF / UWB sweeps) cubical complexes avoid the triangulation overhead simplicial complexes incur.

### Naming: `LatticeComplex<D>` (canonical) and `CubicalComplex<D>` (alias)

The type ships under **two names** that refer to the same thing:

- **`LatticeComplex<const D: usize>`** — the canonical name. It honestly describes what the struct stores: a regular ℤᴰ lattice (`shape` + `periodic` flags) carrying the cellular decomposition on top. This name also keeps consistent with the rest of the physics vocabulary in this crate — `LatticeGaugeField<G, D, M, R>`, "lattice gauge theory", etc.
- **`CubicalComplex<const D: usize>`** — a `pub type` alias on `LatticeComplex<D>`. Provided for callers reading from the algebraic-topology side of the literature (Kaczynski–Mischaikow–Mrozek, Edelsbrunner–Harer) where this structure is universally called a *cubical complex*.

The two names refer to the same structure and are equally canonical — each emphasizes a different mathematical tradition (substrate vs. cellular decomposition). Use whichever fits the surrounding text. An equivalent dual-name pairing ships for the elementary cell: `LatticeCell<D>` ↔ `CubicalCell<D>`. No other aliases exist: types such as `DualLatticeComplex<D>` and `LatticeComplexWitness<D>` have only their canonical name because there is no genuinely distinct cubic-specialization reading for them.

**Scope note.** `LatticeComplex<D>` represents specifically the cubical complex of a **regular** ℤᴰ grid (with optional periodicity per axis). It is not a sparse cubical complex (only-active-cubes) or an irregular one (cubes of varying side lengths) — those are deferred follow-ups. The naming choice reflects this: a "lattice complex" makes the regular-grid restriction explicit; "cubical complex" alone could mislead readers into expecting the general case.

### What the type provides

`LatticeComplex<D>` implements `ChainComplex`, so all the boundary / coboundary / Betti / chain-complex machinery available to `SimplicialComplex<T>` is also available to it. `LatticeCell<D>` encodes elementary k-cubes as a base position + orientation bitmask (bit `i` set ⇒ the cell extends in dimension `i`); the standard cubical boundary operator is implemented on the cell directly.

The physics-domain names — `LatticeGaugeField`, `LatticeGaugeFieldWitness`, `LinkVariable`, the `gauge_field_lattice/` module — are retained as-is (they refer to lattice gauge theory, not the topological structure).

### Metric

`Manifold<LatticeComplex<D>, F>` carries an optional `CubicalReggeGeometry<D>` — the cubical analogue of `ReggeGeometry<T>`, paralleling Regge calculus on simplicial complexes. The type encodes edge lengths at four levels of uniformity (unit edge, isotropic, per-axis, per-edge) plus optional Lorentzian / timelike axis flags. The Stage C scope ships **edge-length storage, access, and signature computation**; derived geometric quantities (cell volumes, deficit angles, Hodge ⋆, per-cell metric tensors) are forward-looking and tracked as a follow-up.

---

## 🗺️ Neighborhood Strategies

Many cellular-automaton-style operations (heat diffusion, sensor fusion, image filters) need to enumerate the neighbors of each cell. `Neighborhood<K: ChainComplex>` is a static-dispatch strategy trait — strategies are zero-sized structs, the iterator is GAT-backed, the call monomorphizes to nothing.

The strategies split cleanly along the **chain-complex-generic** / **grid-specific** boundary:

| Strategy | Implemented for | Definition |
|---|---|---|
| `FaceAdjacent` | any `K: ChainComplex` | Top cells sharing a `(max_dim − 1)`-face, derived from ∂. |
| `CofaceAdjacent` | any `K: ChainComplex` | `(max_dim)`-cells containing the target `(max_dim − 1)`-cell as a face, derived from δ. |
| `VonNeumann` | `CubicalComplex<D>` only | Top cubes face-adjacent on the grid (coincides with `FaceAdjacent` on top cells, uses grid coordinates for an O(D) fast path). |
| `Moore` | `CubicalComplex<D>` only | Top cubes at Chebyshev distance ≤ 1 (up to `3^D − 1`). |
| `KRing<const K: usize>` | `CubicalComplex<D>` only | Top cubes at Chebyshev distance ≤ K. |

`Moore` and `KRing` are **deliberately not implemented** for simplicial complexes — there is no principled definition of "Moore neighborhood" without the regular-grid coordinate structure. The compiler enforces this: `Moore.neighbors(&simplicial_complex, ...)` fails to compile.

Strategies are passed by value to `Manifold::neighbors`:

```rust
use deep_causality_topology::{CubicalComplex, Manifold, Moore};

let complex = CubicalComplex::<2>::open([32, 32]);
let manifold: Manifold<CubicalComplex<2>, f64> = Manifold::from_cubical(complex, data, 0);
for n in manifold.neighbors(Moore, cell_id) {
    // ... read manifold.data()[n] and compute the next state
}
```

Users add their own strategies (anisotropic LIDAR cones, half-space RF, etc.) by implementing `Neighborhood<K>` for their own zero-sized types — no fork of the crate needed.

See `examples/cubical_heat_diffusion.rs` for a worked example: a 16×16 cubical grid evolved 10 explicit-Euler steps under a Moore-neighborhood discrete-Laplacian stencil.

---

## 🧭 Manifold Aliases and Witnesses

`Manifold<K: ChainComplex, F>` is generic over the underlying complex. Two aliases ship for the simplicial fast path:

| Alias | Resolves to | Purpose |
|---|---|---|
| `SimplicialManifold<C, F>` | `Manifold<SimplicialComplex<C>, F>` | The textbook simplicial manifold. Existing simplicial code uses this. |
| `SimplicialManifoldWitness<C>` | `ManifoldWitness<C>` | HKT entry point for simplicial manifolds. Carries the existing `Functor` / `Monad` / `Applicative` / `CoMonad` impls. |
| `GenericManifoldWitness<K>` | — (new struct) | HKT entry point for arbitrary-`K` manifolds (e.g. cubical). Ships with `HKT` and `Functor`; `Monad` / `Applicative` / `CoMonad` are deferred because they need default-constructible complex bounds. |

---

## Summary

| Structure | Purpose |
|-----------|---------|
| `Simplex` | Atomic geometric unit (Point, Line, Triangle). |
| `SimplicialComplex` | Collection of simplices with computed topology (∂, δ, ⋆). |
| `LatticeComplex<D>` | Canonical name: D-dimensional cubical complex on a regular ℤᴰ lattice. Implements `ChainComplex`. |
| `CubicalComplex<D>` | Textbook alias for `LatticeComplex<D>`. |
| `LatticeCell<D>` | Canonical name: elementary k-cube within a `LatticeComplex<D>`. |
| `CubicalCell<D>` | Textbook alias for `LatticeCell<D>`. |
| `Manifold<K, F>` | Validated complex + data field + optional `K::Metric` + cursor. |
| `SimplicialManifold<C, F>` | Alias for `Manifold<SimplicialComplex<C>, F>`. |
| `ReggeGeometry<T>` | Simplicial Regge metric (edge lengths, deficit angles). |
| `CubicalReggeGeometry<D>` | Cubical analogue of Regge geometry: edge-length data on a lattice complex, with unit / isotropic / per-axis / per-edge representations and optional Lorentzian axis flags. |
| `Neighborhood<K>` | Static-dispatch strategy trait for cell-neighborhood queries. |
| `FaceAdjacent` / `CofaceAdjacent` | Chain-complex-generic neighborhood strategies. |
| `VonNeumann` / `Moore` / `KRing<const K>` | Grid-only neighborhood strategies for `CubicalComplex<D>`. |
| `Point Cloud` | Raw spatial data generator (e.g., Triangulation). |
