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

## Summary

| Structure | Purpose |
|-----------|---------|
| `Simplex` | Atomic geometric unit (Point, Line, Triangle). |
| `SimplicialComplex` | Collection of simplices with computed topology (∂, δ, ⋆). |
| `Manifold` | Validated complex + Data Field + Metric/Gravity. |
| `ReggeGeometry` | Discrete curvature and metric storage. |
| `Point Cloud` | Raw spatial data generator (e.g., Triangulation). |
