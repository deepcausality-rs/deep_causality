# General Relativity Gauge Field Pipeline

This example demonstrates **General Relativity (GR)** as an SO(3,1) Lorentz gauge theory
using the **Causal Monad** (`PropagatingEffect`) for type-safe, modular composition of
physics stages.

## Overview: General Relativity in 5 Lines

The following 5 lines of code encapsulate the essential workflow of modern numerical relativity:

```rust
let result = initial_stage_create_schwarzschild()
    .bind_or_error(stage_curvature_invariants, "Curvature computation failed")
    .bind_or_error(stage_geodesic_analysis, "Geodesic analysis failed")
    .bind_or_error(stage_adm_formalism, "ADM formalism failed")
    .bind_or_error(stage_horizon_detection, "Horizon detection failed");
```

## The Gravitas of each Stage

While the code looks simple, each stage performs a fundamental operation that defines our understanding of the universe:

### 1. Building a Black Hole (Initial Stage)
We start by constructing the fabric of space-time itself. In this example, we model a Schwarzschild black hole that is a region of space so dense that even light cannot escape. We're defining the gravitational arena where everything happens.

### 2. Measuring the Warp (Curvature Invariants)
Einstein's greatest insight was that gravity is the curvature of space-time itself. In this stage, we measure exactly how "warped" the universe is at our specific location. By checking the **Ricci Scalar**, we confirm we are in a vacuum—there is no matter here, only the pure gravitational influence of the distant mass.

### 3. The Path of Least Resistance (Geodesic Analysis)
Everything in the universe follows the "straightest possible path" through curved space, called a **geodesic**. Here, we calculate how objects (and light) move. We also calculate **Tidal Forces**, which are the physical stretching effect felt by an explorer, and we measure **Time Dilation**, where time literally slows down because of the strenghtening gravity as you get closer to the black hole.

### 4. Slicing through Time (ADM Formalism)
To simulate gravity on a computer, we slice the 4D universe into layers of 3D space, much like frames in a movie. This stage uses the **Hamiltonian Constraint** to check if our slices are mathematically correct. It ensures that the laws of physics are preserved as space-time evolves from one moment to the next.

### 5. Finding the Event Horizon (Horizon Detection)
Finally, we locate the boundaries that define a black hole. We find the **Event Horizon**, the point of no return; the **Photon Sphere**, where light itself orbits in a circle; and the **ISCO**, the last safe harbor where a planet or moon can orbit without being dragged into the black hole.

---

## Key Physics Concepts

### Schwarzschild Metric
The spacetime geometry of a non-rotating mass:
```
ds² = -(1 - r_s/r)dt² + (1 - r_s/r)^{-1}dr² + r²dΩ²
```
where r_s = 2GM/c² is the Schwarzschild radius.

### Detailed Invariants
| Invariant    | Formula                     | Physical Meaning               |
|--------------|-----------------------------|--------------------------------|
| Kretschmann  | K = R_μνρσ R^μνρσ = 48M²/r⁶ | Absolute curvature strength    |
| Ricci Scalar | R = g^μν R_μν = 0           | Matter coupling (0 for vacuum) |

### ADM 3+1 Variables
- **Lapse (α)**: How fast proper time ticks relative to coordinate time.
- **Shift (β)**: How coordinates "drift" over the spatial slice.
- **Extrinsic Curvature (K)**: How the 3D slice is "bent" inside the 4D spacetime.

---

## Running the Example

```bash
cargo run --example gauge_gr -p physics_examples
```

## Design Pattern: The Causal Monad

This example showcases the power of the **Causal Monad** (`PropagatingEffect`). By using `.bind_or_error()`, we treat the complex mathematics of General Relativity as a simple pipeline of operations.

### Why this matters:
1. **Type-Safe**: Each stage is guaranteed to have the physical data it needs from the previous one.
2. **Error Handling**: If a computation becomes physically impossible (like trying to measure time inside a singularity), the pipeline stops and explains why, without crashing.
3. **Modularity**: You could swap the "Black Hole" stage for a "Neutron Star" or "Gravitational Wave" stage without changing any of the analysis code.


## GR Operations Used

| Operation              | Method                               | Description         |
|------------------------|--------------------------------------|---------------------|
| Schwarzschild radius   | `GR::schwarzschild_radius()`         | r_s = 2GM/c²        |
| Kretschmann scalar     | `gr.kretschmann_scalar()`            | Curvature invariant |
| Ricci scalar           | `gr.ricci_scalar()`                  | Ricci contraction   |
| Geodesic deviation     | `gr.geodesic_deviation()`            | Tidal forces        |
| Hamiltonian constraint | `AdmState::hamiltonian_constraint()` | ADM constraint      |
| Mean curvature         | `AdmState::mean_curvature()`         | Trace of K_ij       |

## Related Examples

- [`gauge_qed`](../gauge_qed/) — Electromagnetic field analysis
- [`gauge_weak_force`](../gauge_weak_force/) — SU(2) weak interaction
- [`gauge_electroweak`](../gauge_electroweak/) — Electroweak unification

## References

- Misner, Thorne, Wheeler, *Gravitation*, Chapters 23, 31 (Schwarzschild, ADM)
- Wald, *General Relativity*, Chapter 6 (Curvature)
- [deep_causality_physics::GrOps](../../deep_causality_physics/src/theories/gr/gr_ops.rs)
