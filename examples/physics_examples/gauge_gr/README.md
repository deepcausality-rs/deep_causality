# General Relativity Gauge Field Pipeline

## Run the example

```bash
RUSTFLAGS='-C target-cpu=native'  cargo run --example gauge_gr --release
```

This example treats **General Relativity (GR)** as an SO(3,1) Lorentz gauge theory and
composes the analysis of a Schwarzschild black hole with the **Causal Monad** (`CausalFlow`).

## Overview: General Relativity in 5 Lines

Five lines chain the workflow of numerical relativity:

```rust
let result = initial_stage_create_schwarzschild()
    .bind_or_error(stage_curvature_invariants, "Curvature computation failed")
    .bind_or_error(stage_geodesic_analysis, "Geodesic analysis failed")
    .bind_or_error(stage_adm_formalism, "ADM formalism failed")
    .bind_or_error(stage_horizon_detection, "Horizon detection failed");
```

## The Gravitas of each Stage

Each stage performs one operation of the analysis:

### 1. Building a Black Hole (Initial Stage)
The first stage constructs the spacetime: a Schwarzschild black hole, a region so dense that even light cannot escape.

### 2. Measuring the Warp (Curvature Invariants)
Gravity is the curvature of spacetime. This stage measures how "warped" spacetime is at the probe's location with the **Kretschmann scalar**. The **Ricci Scalar** is zero, which marks a vacuum: no matter at this point, only the gravitational influence of the distant mass.

### 3. The Path of Least Resistance (Geodesic Analysis)
Free objects and light follow the "straightest possible path" through curved spacetime, a **geodesic**. This stage computes the **Tidal Forces**, the stretching an explorer would feel, from the geodesic deviation, and the **Time Dilation**: clocks slow as gravity strengthens closer to the black hole.

### 4. Slicing through Time (ADM Formalism)
Numerical relativity slices 4D spacetime into layers of 3D space, like frames in a movie. This stage checks the slice against the **Hamiltonian Constraint**, which every valid slice must satisfy as spacetime evolves.

### 5. Finding the Event Horizon (Horizon Detection)
The last stage locates the boundaries of the black hole: the **Event Horizon**, the point of no return; the **Photon Sphere**, where light orbits in a circle; and the **ISCO** (innermost stable circular orbit), the closest radius at which matter can orbit stably.

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

`CausalFlow` chains the stages with `.bind_or_error()`, so the mathematics of General Relativity runs as a pipeline of operations.

### Why this matters:
1. **Type-Safe**: Each stage receives the physical data it needs from the previous one.
2. **Error Handling**: If a computation becomes physically impossible (like measuring time inside a singularity), the pipeline stops and reports why, without crashing.
3. **Modularity**: A "Neutron Star" or "Gravitational Wave" stage can replace the "Black Hole" stage without changes to the analysis code.


## GR Operations Used

| Operation              | Method                               | Description         |
|------------------------|--------------------------------------|---------------------|
| Schwarzschild radius   | `GR::schwarzschild_radius()`         | r_s = 2GM/c²        |
| Kretschmann scalar     | analytic, `48M²/r⁶`                  | Curvature invariant |
| Ricci scalar           | vacuum, `R = 0`                      | Ricci contraction   |
| Geodesic deviation     | `gr.geodesic_deviation_si()`         | Tidal forces        |
| Hamiltonian constraint | `AdmState::hamiltonian_constraint()` | ADM constraint      |
| Mean curvature         | `AdmState::mean_curvature()`         | Trace of K_ij       |

## Related Examples

- [`gauge_qed`](../gauge_qed/): Electromagnetic field analysis
- [`gauge_weak_force`](../gauge_weak_force/): SU(2) weak interaction
- [`gauge_electroweak`](../gauge_electroweak/): Electroweak unification

## References

- Misner, Thorne, Wheeler, *Gravitation*, Chapters 23, 31 (Schwarzschild, ADM)
- Wald, *General Relativity*, Chapter 6 (Curvature)
- [deep_causality_physics::GrOps](../../deep_causality_physics/src/theories/gr/gr_ops.rs)
