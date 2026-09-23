# GRMHD: General Relativistic Magnetohydrodynamics

This example couples General Relativity to Magnetohydrodynamics through DeepCausality's monadic composition: a curvature quantity the GR solver computes decides which algebra the MHD solver runs in.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p physics_examples --example grmhd_example
```

---

## Engineering Value

Simulating extreme environments (Black Holes, Neutron Stars) requires coupling:
- **General Relativity**: Gravity and Spacetime Curvature
- **Magnetohydrodynamics**: Plasma and Electromagnetic Fields

The simulation adapts its mathematical foundation to the physical conditions, so it stays valid in extreme regimes.

---

## Causal Chain

```text
[Step 1] GR Solver          → Schwarzschild metric → Kretschmann scalar, tidal acceleration
                                       ↓
[Step 2] Coupling Layer     → Select Metric (Euclidean vs Minkowski)
                                       ↓
[Step 3] MHD Solver         → Lorentz force density (F = J ∧ B)
                                       ↓
[Step 4] Feedback           → EM stress-energy T^tt on the Schwarzschild metric
                                       ↓
[Step 5] Stability Analysis → Confinement Status
```

`CausalFlow::value` starts the chain, one `.next` runs each stage, and `finish` returns the final state or the error a stage short-circuited with.

---

## Physics Components

All quantities are in geometric units, `G = c = 1`; the tidal acceleration is also reported in `m/s^2`.

### GR Solver (`CausalTensor`)

Builds the Schwarzschild metric with `generate_schwarzschild_metric`. In the vacuum exterior the Ricci scalar is zero, so the curvature invariant that carries the tidal physics is the Kretschmann scalar `K = 48 M^2 / r^6`. The stage also computes the radial tidal acceleration across the plasma column, `a = 2 M L / r^3`.

### Coupling Layer

Selects the Clifford metric from the tidal acceleration:
- **Above the threshold (`1e-12`)**: Minkowski(4), relativistic 4D spacetime
- **At or below the threshold**: Euclidean(3), classical 3D space

### MHD Solver (`CausalMultiVector`)

Computes the Lorentz force density with `lorentz_force` as the wedge product `F = J ∧ B` of the current and the magnetic field, in the algebra the coupling layer selected.

### Feedback

Carries the observer's `B` into coordinates, computes the EM stress-energy with `energy_momentum_tensor_em`, and projects `T^tt` back onto the observer. The result must equal `B^2 / 2`; the run checks this identity and exits nonzero if it fails.

### Stability Analysis

Compares the curvature the plasma sources, `8 pi rho_EM`, with the curvature the hole imposes, `sqrt(K)`:
- **Negative force**: reversed confinement, the force points out of the column
- **`8 pi rho_EM > sqrt(K)`**: magnetically dominated
- **Otherwise**: tidally dominated

---

## Key Insight

Tensors and multivectors compose monadically into one multi-physics chain. Each stage transforms the state through a pure function.
