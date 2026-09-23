# Maxwell's Unification: Causaloid Example

This example derives the electric and magnetic fields of a plane wave as one `CausalMultiVector` bivector from a single vector potential, and chains the stages with DeepCausality's monadic composition.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p physics_examples --example maxwell_example
```

---

## Engineering Value

Standard engineering treats the Electric (E) and Magnetic (B) fields as separate vectors and checks their consistency by hand. Geometric Algebra unifies them into one **Electromagnetic Field Bivector (F)** derived from a **Vector Potential (A)**.

### Application: 5G/6G Antenna Design (Phased Arrays)

- Simulate the **Interference Pattern** of the Vector Potential directly on the antenna mesh
- Computing A (4 scalars) is **50% faster** than computing E, B (6 scalars)
- Numerically more stable (no divergence cleaning)

---

## Physics Background

### The Vector Potential Formulation

In place of separate E and B, the example uses:

```
F = ∇A (Geometric Product)
```

Where:
- **A**: 4-Vector Potential (φ, A_x, A_y, A_z)
- **∇**: Spacetime Gradient Operator
- **F**: Electromagnetic Field Bivector

### Extracted Components

The geometric product yields:
- **Scalar (Grade 0)**: Divergence → Lorenz Gauge Check
- **Bivector e_tx**: Electric Field E
- **Bivector e_xz**: Magnetic Field B

### Plane Wave Example

For a linearly polarized plane wave moving in the Z-direction:
```
A = (0, cos(ω(t-z)), 0, 0)
```

The run evaluates the potential over `Dual` numbers, so automatic differentiation yields `E_x = -dA_x/dt` and `B_y = dA_x/dz` exactly. It then checks:
- `|E| = |B|` (characteristic of light waves)
- `Divergence ≈ 0` (Lorenz Gauge satisfied)
- `|S| = |E||B|` for the Poynting flux `S = E × B`
- the AD fields against the closed form

A failed check exits the process with a nonzero status.

---

## Causal Chain

```text
PlaneWaveConfig → Potential(A) → EM Field(F = ∇A) → Gauge Check → Results
```

Each step is a pure function; `CausalFlow::bind` chains the potential, the field bivector and the Poynting flux.

## Reference

For more on Geometric Algebra in electromagnetism, see:
- Hestenes, D. "Spacetime Algebra" (Gordon and Breach, 1966)
