# Gauge Theories & Geometric Algebra

In `deep_causality_physics`, we adopt a modern, unified approach to theoretical physics by leveraging **Geometric
Algebra (GA)** and **Gauge Fields** as our foundational abstractions. This document explains why we chose this
architecture and how it unifies diverse physical phenomena.

---

## 🏗️ The Unification Problem

In standard physics simulations, different theories are often implemented with completely different data structures:

* **Gravity (GR)**: Uses metric tensors $g_{\mu\nu}$ and Riemann curvature tensors $R^\mu_{\nu\rho\sigma}$.
* **Electromagnetism (EM)**: Uses electric $\vec{E}$ and magnetic $\vec{B}$ vectors, or the Faraday tensor $F_{\mu\nu}$.
* **Particle Physics (Weak/Strong)**: Uses Lie algebra generators, spinors, and gauge potentials $A_\mu$.

This fragmentation makes it difficult to simulate interactions *between* theories (e.g., gravity affecting an
electromagnetic field) without writing bespoke glue code for every pair of interactions.

## 🔑 The Solution: Gauge Fields & Geometric Algebra

We unify these theories by treating them all as **Gauge Theories** defined over a manifold, powered by **Geometric
Algebra**.

### 1. Everything is a Gauge Field

The `GaugeField<G>` abstraction (from `deep_causality_topology`) serves as the universal container for all physical
fields. It is parameterized by a `GaugeGroup` (G), which defines the symmetry of the theory:

| Theory                 | Gauge Group    | Physics Impl       | Description                                               |
|------------------------|----------------|--------------------|-----------------------------------------------------------|
| **Electromagnetism**   | `U(1)`         | `EM`               | Abelian gauge theory. Phase symmetry.                     |
| **Weak Force**         | `SU(2)`        | `WeakField`        | Non-Abelian. Isospin symmetry.                            |
| **Electroweak**        | `SU(2) x U(1)` | `ElectroweakField` | Unified EM + Weak. Symmetry breaking via Higgs.           |
| **General Relativity** | `Lorentz`      | `GR`               | Gauge theory of the Lorentz group (Gravity as curvature). |

By using a shared `GaugeField` struct, we gain access to universal topological operations:

* **Field Strength Computation**: $F = dA + A \wedge A$ (Works for EM, Weak, and Gravity)
* **Covariant Derivatives**: $D_\mu = \partial_\mu + [A_\mu, \cdot]$
* **Bianchi Identities**: $DF = 0$
* **Parallel Transport**: Moving vectors along a path while respecting curvature.

### 2. Geometric Algebra (GA) as the Engine

Geometric Algebra (Clifford Algebra) generalizes complex numbers and quaternions to $n$-dimensions. It allows us to:

* **Multiply Vectors**: $uv = u \cdot v + u \wedge v$ (Dot product + Wedge product).
* **Unify Spacetime**: Treat space and time on equal footing in 4D spacetime.
* **Coordinate Independence**: Write equations that are true regardless of the basis choice.

In `deep_causality_physics`, we use `CausalMultiVector` to represent physical quantities. This means an electromagnetic
field isn't just a list of numbers; it's a **bivector** field that inherently encodes the rotational properties
of $\vec{E}$ and $\vec{B}$.

---

## 🚀 Practical Benefits

This architecture provides three major advantages:

### A. Code Re-use via Witness Types

Instead of rewriting the "Field Strength" calculation for every theory, we implement it **once** in the topology layer
using **HKT Witness Types** (`GaugeFieldWitness`).

* **Electromagnetism**: Uses `field_strength_from_eb_vectors` (Topology)
* **Weak Force**: Uses `compute_field_strength_non_abelian` (Topology)
* **Gravity**: Uses `expand_lie_to_riemann` (Topology)

This ensures that a bug fix in the topology layer improves *all* physics theories simultaneously.

### B. Topological Consistency

Because all fields are grounded in the same topological manifold (`deep_causality_topology`), we can guarantee
consistency:

* **Gap Closure**: We recently identified and closed gaps where physics implementation diverged from topological
  defintions (e.g., [Weinberg Mixing](./src/theories/electroweak/electroweak_impl.rs)
  and [Kretschmann Scalar](./src/theories/general_relativity/gr_ops_impl.rs)).
* **Precision**: Topological operations often yield higher precision by preserving geometric invariants that standard
  floating-point arithmetic might violate.

### C. Future-Proofing

New theories (e.g., QCD with `SU(3)`) can be added simply by defining a new `GaugeGroup`. The existing machinery for
field strength, transport, and curvature will work "out of the box."

---

## 🔬 Configurable Precision

One of the unique features of `deep_causality_physics` is that all Gauge Theories are generic over the floating-point
type. This allows you to trade off between speed and extreme precision without changing your physics code.

Different precision levels are supported via type aliases (see `deep_causality_physics/src/theories/alias/mod.rs`):

| Float Type    | Precision          | Use Case                                                  |
|---------------|--------------------|-----------------------------------------------------------|
| `f32`         | ~7 decimal digits  | **Game Physics / Real-time Viz** (Max Speed)              |
| `f64`         | ~16 decimal digits | **Standard Engineering / Simulations** (Default)          |
| `DoubleFloat` | ~31 decimal digits | **Cosmology / Quantum Field Theory** (Error Minimization) |

### Example: Switching Precision

As shown in `examples/physics_examples/gauge_gr/main.rs`, you can switch the entire simulation precision by changing a
single type alias:

```rust
use deep_causality_physics::{GR, EM};
use deep_causality_num::DoubleFloat;

// 1. Standard Precision Analysis
type StandardGR = GR<f64>;

// 2. High-Precision Cosmology (128-bit sim)
// Useful for integrating geodesics near singularities where errors accumulate
type HighPrecisionGR = GR<DoubleFloat>;

// 3. Low-Precision/Fast Visualization
type GamePhysicsEM = EM<f32>;
```

All internal operations—matrix inversions, wedge products, and curvature contractions—will automatically use the
specified precision.


---

## 📚 The Hierarchy

1. **Topology Layer (`deep_causality_topology`)**:
    * Defines `GaugeField<G>`, `Manifold`, `CurvatureTensor`.
    * Implements universal algorithms (`gauge_rotation`, `kretschmann_scalar_with_metric`).

2. **Physics Kernel Layer (`src/*/*.rs`)**:
    * Implements isolated equations (`einstein_tensor_kernel`, `lorentz_force_kernel`).
    * Used when you just need a number, not a simulation.

3. **Physics Theory Layer (`src/theories/*`)**:
    * **The "Sweet Spot"**: Combines 1 & 2.
    * Uses topology for structure and kernels for specific empirical values.
    * *Example*: The `GR` module uses topology's `CurvatureTensor` to compute the Einstein tensor, but physics kernels
      to compute geodesic trajectories.

---

> **Summary**: Use **Kernels** for isolated equations. Use **Theories** for full simulations. The Theories are powered
> by **GA** and **Gauge Fields**.

---

