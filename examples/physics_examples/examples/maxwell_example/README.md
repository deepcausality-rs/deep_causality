# Maxwell's Unification: Causaloid Example

This example demonstrates how to model Maxwell's electromagnetic field unification using DeepCausality's monadic composition with `CausalMultiVector`.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p physics_examples --example maxwell_example
```

---

## Engineering Value

In standard engineering, Electric (E) and Magnetic (B) fields are treated as separate vectors, requiring manual consistency checks. In Geometric Algebra, they are unified into a single **Electromagnetic Field Bivector (F)** derived from a **Vector Potential (A)**.

### Application: 5G/6G Antenna Design (Phased Arrays)

- Simulate the **Interference Pattern** of the Vector Potential directly on the antenna mesh
- Calculate A (4 scalars) is **50% faster** than calculating E, B (6 scalars)
- Numerically more stable (no divergence cleaning)

---

## Physics Background

### The Vector Potential Formulation

Instead of working with E and B separately, we use:

```
F = ∇A (Geometric Product)
```

Where:
- **A**: 4-Vector Potential (φ, A_x, A_y, A_z)
- **∇**: Spacetime Gradient Operator
- **F**: Electromagnetic Field Bivector

### Extracted Components

The geometric product automatically produces:
- **Scalar (Grade 0)**: Divergence → Lorenz Gauge Check
- **Bivector e_tx**: Electric Field E
- **Bivector e_xz**: Magnetic Field B

### Plane Wave Example

For a linearly polarized plane wave moving in the Z-direction:
```
A = (0, cos(ω(t-z)), 0, 0)
```

The physics verification confirms:
- `|E| = |B|` (characteristic of light waves)
- `Divergence ≈ 0` (Lorenz Gauge satisfied)

---

## Causal Chain

```text
PlaneWaveConfig → Potential(A) → EM Field(F = ∇A) → Gauge Check → Results
```

Each step is a pure function composed monadically via `PropagatingEffect::bind`.

## Reference

For more on Geometric Algebra in electromagnetism, see:
- Hestenes, D. "Spacetime Algebra" (Gordon and Breach, 1966)
