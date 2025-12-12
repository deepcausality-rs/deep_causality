# Material Science Examples

This directory contains examples applying **DeepCausality** to Material Science and Metamaterials. These examples demonstrate advanced physics simulations using the framework's topology, multivector algebra, and causal intervention capabilities.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p material_examples --example <example_name>
```

---

## Examples Overview

| Example | Domain | Description |
|---------|--------|-------------|
| [hyperlens](examples/hyperlens/README.md) | Metamaterials | Super-resolution imaging using hyperbolic dispersion |
| [topological_insulator](examples/topological_insulator/README.md) | Quantum Materials | Chern number calculation for topological phase classification |
| [structural_health_monitor](examples/structural_health_monitor/README.md) | Smart Materials | Decentralized monitoring with autonomous causal interventions |

---

## Common Patterns

### 1. `Metric` for Material Properties
Different materials have different "signatures" that determine how waves propagate:
*   **Euclidean**: Standard vacuum/dielectric
*   **Minkowski**: Relativistic spacetime
*   **Generic(p, q, r)**: Anisotropic/Hyperbolic metamaterials

```rust
use deep_causality_multivector::Metric;

let vacuum = Metric::Euclidean(3);           // (+++)
let hyperbolic = Metric::Generic { p: 1, q: 2, r: 0 }; // (+−−)
```

### 2. `Manifold<T>` & `Graph<T>` for Topology
*   **Manifold**: Continuous surfaces with differential structure (hyperlens geometry).
*   **Graph**: Discrete networks (structural lattices, molecular bonds).

### 3. `Intervenable` for Active Materials
Smart materials that can **autonomously respond** to stimuli use the `Intervenable` trait to formally separate:
*   **Observation** (what is happening)
*   **Intervention** (what we force to happen)
*   **Counterfactual** (what would have happened otherwise)

---

## Run Commands

| Example | Command |
|---------|---------|
| Hyperlens | `cargo run -p material_examples --example hyperlens_example` |
| Topological Insulator | `cargo run -p material_examples --example topological_insulator_example` |
| Structural Health Monitor | `cargo run -p material_examples --example structural_health_monitor_example` |

---

## Crates Used

*   **`deep_causality_multivector`**: Geometric Algebra for anisotropic field representations.
*   **`deep_causality_topology`**: `Graph`, `Manifold`, `SimplicialComplex` for structural modeling.
*   **`deep_causality_num`**: High-precision Complex number arithmetic for quantum calculations.
*   **`deep_causality_core`**: `PropagatingEffect`, `Intervenable` for monadic state and interventions.

---

## Adding New Examples

1.  Create a new directory under `examples/`: `examples/<your_example>/`
2.  Add `main.rs` and `README.md`
3.  Register in `Cargo.toml`:
    ```toml
    [[example]]
    name = "your_example"
    path = "examples/<your_example>/main.rs"
    ```
