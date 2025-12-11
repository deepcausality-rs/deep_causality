# DeepCausality Physics: The Causal Universe

`deep_causality_physics` provides a comprehensive suite of physics kernels covering domains from Classical Mechanics to Quantum Field Theory. It is designed not just to *calculate* values, but to *propagate effects* within a causal system.

---

## 🌌 Philosophy: Physics as Information

In DeepCausality, physical laws are treated as **Causal Functions**.
*   **Input**: The state of the universe at $t$ (represented by `Manifold` or `HilbertState`).
*   **Function**: The physical law (e.g., Schrödinger equation, Maxwell's equations).
*   **Output**: The state of the universe at $t+1$ (wrapped in a `PropagatingEffect`).

This allows us to model physical systems where:
1.  **Context Matters**: Constants like $c$ or $G$ can be contextual.
2.  **Errors Propagate**: Numerical instabilities or unphysical states are tracked monadically.
3.  **Causality is Explicit**: We can trace *why* a state collapsed or a particle moved.

---

## ⚛️ Key Domains

The crate is organized into modular domains:

| Domain | Key Concepts |
|--------|--------------|
| **Quantum** | Gates (Hadamard, CNOT), Hilbert Spaces, Wave Functions. |
| **Relativity** | Spacetime Interval, Lorentz Transformations, Metrics. |
| **Thermodynamics** | Heat Diffusion, Entropy, Enthalpy. |
| **Electromagnetism**| Maxwell's Equations, Fields. |
| **Nuclear** | Decay, Cross-sections. |
| **Astro** | Orbital mechanics, Redshift. |

---

## 🔗 Causal Integration ("Wrappers")

Most physics functions come in two flavors:
1.  **Pure Functions**: Standard Rust functions returning `f64` or `Complex<f64>`.
2.  **Causal Wrappers**: Functions returning `PropagatingEffect<T>`.

**Example:**
The `quantum::wrappers::born_probability` function doesn't just return a probability. It returns a `PropagatingEffect` that:
*   Checks normalization conditions.
*   Logs the computation.
*   Returns an `Error` if the state is invalid (non-unitary).

---

## 📐 Geometric Integration

The physics engine relies heavily on `deep_causality_topology` and `deep_causality_tensor`:

*   **Field Theories** (Gravity, Heat) operate on `Manifold<T>`.
*   **Quantum Mechanics** operates on `CausalMultiVector` (Geometric Algebra) or `CausalTensor` (Linear Algebra).
*   **Relativity** uses `ReggeGeometry` for discrete curvature calculations.

---

## 🛠 Usage Example

Modeling a Quantum Circuit:

```rust
// 1. Define State
let psi = HilbertState::new_qubit(Complex::one(), Complex::zero()); // |0>

// 2. Apply Gates (Monadically)
let result = PropagatingEffect::pure(psi)
    .bind(|s| apply_hadamard(s))       // Superposition
    .bind(|s| apply_phase_shift(s, PI)); // Phase rotation

// 3. Measure
let probability = born_probability(result.extract()?);
```

---

## Summary

`deep_causality_physics` bridges the gap between numerical simulation and causal reasoning. It provides the "Laws of Physics" as composable, safe, and traceable components for the DeepCausality engine.
