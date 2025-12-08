# GRMHD: General Relativistic Magnetohydrodynamics

This example demonstrates a "Multi-Physics Monad" approach for coupling General Relativity with Magnetohydrodynamics using DeepCausality's monadic composition.

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

This architecture ensures that the simulation adapts its mathematical foundation to the physical conditions, preventing validity errors in extreme regimes.

---

## Causal Chain

```text
[Step 1] GR Solver          → Spacetime Metric (g_uv) → Curvature Intensity
                                       ↓
[Step 2] Coupling Layer     → Select Metric (Euclidean vs Minkowski)
                                       ↓
[Step 3] MHD Solver         → Lorentz Force (F = J · B)
                                       ↓
[Step 4] Stability Analysis → Confinement Status
```

---

## Physics Components

### GR Solver (Tensor Monad)

Uses `CausalTensor` and the Applicative HKT to compute the Einstein tensor:

```rust
G_uv ≈ R * g_uv  (Simplified Einstein Field Equations)
```

### Coupling Layer

Dynamically selects the metric based on curvature intensity:
- **High Curvature (> 0.05)**: Minkowski(4) - Relativistic 4D spacetime
- **Low Curvature (≤ 0.05)**: Euclidean(3) - Classical 3D space

### MHD Solver (MultiVector Monad)

Uses `CausalMultiVector` to compute the Lorentz force:

```rust
F = J · B  (Inner product of current and magnetic field)
```

### Stability Analysis

Interprets the force direction:
- **Negative Force**: Relativistic Reversal - Frame dragging effect
- **Positive Force**: Standard stable confinement

---

## Key Insight

The example demonstrates how different mathematical structures (Tensors, MultiVectors) can be composed monadically to model complex multi-physics systems. Each step in the causal chain transforms state through pure functions, maintaining referential transparency while handling sophisticated physics.
