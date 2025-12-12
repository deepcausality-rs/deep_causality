# Avionics Examples

This directory contains examples demonstrating high-assurance avionics software patterns using the `deep_causality` framework. These examples focus on **Safety Critical Systems**, **Guidance, Navigation & Control (GNC)**, and **Autonomous Interventions**.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p avionics_examples --example <example_name>
```

---

## Examples Overview

| Example | Domain | Description |
|---------|--------|-------------|
| [magnav](magnav/README.md) | Navigation | Magnetic Navigation using Causal Particle Filters (Bayesian estimation) |
| [geometric_tcas](geometric_tcas/README.md) | Collision Avoidance | NextGen TCAS using Geometric Algebra collision detection and `Intervenable` safety interlocks |
| [hypersonic_2t](hypersonic_2t/README.md) | Defense/Tracking | Tracking Hypersonic Glide Vehicles (HGV) using Dual-Time (2T) Physics in 6D phase space |

> **See also:** [physics_examples](../physics_examples/README.md) for more Geometric Algebra applications.

---

## Common Patterns

### Safety Interlocks via `Intervenable`

The `geometric_tcas` example demonstrates the **Closed Loop Intervention** pattern. Instead of relying on ad-hoc conditional logic for safety overrides (e.g., auto-pilot engagement), it uses the formal `Intervenable` trait (Pearl's Layer 2).

```rust
// Formal Computational Intervention
let safe_state = effect.intervene(new_vector);
```

This separates the **Natural History** (pilot did nothing) from the **Forced History** (auto-pilot took over), providing a rigorous audit trail for "Black Box" recorders.

### Coordinate-Free Dynamics

Both `geometric_tcas` and `hypersonic_2t` leverage **Geometric Algebra (`deep_causality_multivector`)** to solve dynamics without singular coordinate systems (like Euler angles).

*   **TCAS**: Uses Bivector magnitude $ \|P \wedge V\| $ to calculate impact parameters directly.
*   **Hypersonic**: Uses Conformal Geometric Algebra (CGA) or 6D phase space to linearize complex orbital/hypersonic trajectories.

---

## Crates Used

| Crate | Purpose |
|-------|---------|
| `deep_causality_core` | Causal Monads (`PropagatingEffect`, `Intervenable`) for safety logic |
| `deep_causality_multivector` | Geometric Algebra for kinematics and relativistic physics |
| `deep_causality_tensor` | Tensor operations for map-based navigation |

---

## Adding New Examples

1. Create directory: `examples/<your_example>/`
2. Add `main.rs` with doc comments (`//!` module docs)
3. Add `README.md` following the [standard template](../physics_examples/README.md)
4. Register in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "examples/your_example/main.rs"
   ```
