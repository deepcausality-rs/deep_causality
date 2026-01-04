# Physics Examples

This directory contains examples demonstrating the `deep_causality_physics` crate and related multi-physics capabilities.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p physics_examples --example <example_name>
```

---

## Examples Overview

| Example | Domain | Description |
|---------|--------|-------------|
| [algebraic_scanner](algebraic_scanner/README.md) | Abstract Algebra | Scans Clifford Algebras for complex structure (I²=-1) |
| [bernoulli_flow_network](bernoulli_flow_network/README.md) | Fluid Dynamics | Pipe network with Venturi effect and hydrostatic gain |
| [carnot_cycle_engine](carnot_cycle_engine/README.md) | Thermodynamics | 4-stage heat engine at Carnot efficiency limit |
| [laser_resonator_stability](laser_resonator_stability/README.md) | Optics | Gaussian beam propagation via ABCD matrices |
| [maxwell_example](maxwell/README.md) | Electromagnetism | Maxwell's equations via Geometric Algebra |
| [grmhd_example](grmhd/README.md) | Relativity | General Relativistic Magnetohydrodynamics |
| [geometric_tilt_example](geometric_tilt/README.md) | Robotics/IMU | Sensor fusion with Geometric Algebra |
| [multi_physics_pipeline](multi_physics_pipeline/README.md) | Particle Physics | QFT → Hadronization → Hydro → Detection |
| [quantum_counterfactual](quantum_counterfactual/README.md) | Quantum | Error correction via state rewind |
| [quantum_geometric_tensor](quantum_geometric_tensor/README.md) | Condensed Matter | QGT, quantum metric, and flat-band transport (TBG) |
| [ikkt_matrix_model](ikkt_matrix_model/README.md) | Quantum Gravity | Emergent spacetime from matrix dynamics |
| [gravitational_wave](gravitational_wave/README.md) | Relativity | Regge Calculus on simplicial mesh |

> **See also:** [medicine_examples](../medicine_examples/README.md) for biophysics examples (protein folding, etc.)

---

## Common Patterns

### Monadic Composition

All examples use `PropagatingEffect` or `CausalEffectPropagationProcess` for composing physics operations:

```rust
let result = step1()
    .bind(|state, _, _| step2(state))
    .bind(|state, _, _| step3(state));
```

### Error Handling

Use `bind_or_error` for robust pipelines:

```rust
let result = risky_operation()
    .bind_or_error(|data, _, _| next_step(data), "Error message");
```

### Type-Safe Physics

Physics types enforce invariants:
- `Probability` - Values in [0, 1]
- `EnergyDensity` - Non-negative energy
- `HilbertState` - Normalized quantum states

---

## Crates Used

| Crate | Purpose |
|-------|---------|
| `deep_causality_physics` | Physics kernels (Klein-Gordon, heat diffusion, etc.) |
| `deep_causality_multivector` | Geometric Algebra (CausalMultiVector, HilbertState) |
| `deep_causality_tensor` | Tensor operations (CausalTensor) |
| `deep_causality_topology` | Discrete geometry (SimplicialComplex, ReggeGeometry) |
| `deep_causality_core` | Monadic effects (PropagatingEffect) |

---

## Adding New Examples

1. Create directory: `examples/<your_example>/`
2. Add `main.rs` with doc comments (`//!` module docs)
3. Add `README.md` with:
   - How to run
   - Engineering value
   - Key concepts
   - APIs demonstrated
   - Adaptation suggestions
4. Register in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "examples/your_example/main.rs"
   ```
