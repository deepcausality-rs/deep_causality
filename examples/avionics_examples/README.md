# Avionics Examples

This directory contains examples demonstrating high-assurance avionics software patterns using the `deep_causality` framework. These examples focus on **Safety Critical Systems**, **Guidance, Navigation & Control (GNC)**, and **Autonomous Interventions**.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p avionics_examples --example <example_name>
```

---

## Examples Overview

| Example | Domain | Description                                                                                                                                                                                                                                                                                                                                                                    |
|---------|--------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [magnav](navigation/magnav/README.md) | Navigation | Magnetic Navigation using Causal Particle Filters (Bayesian estimation)                                                                                                                                                                                                                                                                                                        |
| [geometric_tcas](control/geometric_tcas/README.md) | Collision Avoidance | NextGen TCAS using Geometric Algebra collision detection and `AlternatableValue` safety interlocks                                                                                                                                                                                                                                                                                  |
| [hypersonic_2t](control/hypersonic_2t/README.md) | Defense/Tracking | Tracking Hypersonic Glide Vehicles (HGV) using Dual-Time (2T) Physics in 6D phase space                                                                                                                                                                                                                                                                                        |
| [flight_envelope_monitor](control/flight_envelope_monitor/README.md) | Health Monitoring | Three-stage stateful pipeline (sensor collection → bind chain → envelope hypergraph) demonstrating uniform composition through `PropagatingProcess<_, FlightState, AircraftConfig>`                                                                                                                                                                                            |
| [turbulence_flow](cfd/turbulence_flow/README.md) | Turbulence / Chaos | Forecast horizon of a chaotic convective flow (Lorenz, the Rayleigh–Bénard 3-mode truncation): the same `Rk4` march at f32/f64/Float106 isolates roundoff growth and shows precision setting how far ahead a turbulent flow can be trusted                                                                                                                                     |
| [plasma_blackout_corridor](cfd/plasma_blackout/corridor/README.md) | Multiphysics / GNC | One continuous Mach-25 descent through plasma blackout: a tensor-train compressed compressible carrier with a shock-fitted Rankine-Hugoniot inflow strip, evolved-state Park-2T ionization gated against the RAM-C II flight anchor, flow-resolved GNSS denial driving a 17-state ESKF, O(1) counterfactual bank branches with trajectory-derived misses, and a cybernetic envelope gate whose clamped command actually steers the 3-DOF lift. Ten self-verifying gates, one provenance log |
| [plasma_blackout_weather](cfd/plasma_blackout/weather/README.md) | Digital Twin / Dispersion | The table factory for the corridor: six weather conditions as counterfactual worlds alternated from one validated baseline, flown concurrently, reduced to a dispersion table that tracks navigation precision against weather (the blackout window and a tactical-grade IMU thermal departure both move the drift). Seven self-verifying gates, full provenance per row |
| [ins_gnss_blackout](navigation/ins_gnss_blackout/README.md) | Navigation / Timing | INS clock holdover through a GPS-denial blackout (jamming / urban canyon / tunnel) on **real Galileo** data: a grmhd-style regime detector flips GNSS available↔denied, the `alternate_value` loop corrects the INS when up and is *withheld* through the dark, and the relativistic clock kernel is *carried* across the outage — the navigation/timing core of any GPS-denied flight, in one auditable `CausalFlow` |

> **CFD/MMS verification examples moved.** The Taylor–Green MMS, Re-1600 DEC solver, lid-driven cavity, graded-MMS, and cylinder cases are now self-verifying examples of the `deep_causality_cfd` crate under `deep_causality_cfd/verification/`; run them with `cargo run -p deep_causality_cfd --example <name>_verification`.
>
> **See also:** [physics_examples](../physics_examples/README.md) for more Geometric Algebra applications.

---

## Common Patterns

### Safety Interlocks via `AlternatableValue`

The `geometric_tcas` example demonstrates the **Closed Loop Intervention** pattern. Instead of relying on ad-hoc conditional logic for safety overrides (e.g., auto-pilot engagement), it uses the formal `AlternatableValue` trait (Pearl's Layer 2).

```rust
// Formal Computational Intervention
let safe_state = effect.alternate_value(new_vector);
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
| `deep_causality_core` | Causal Monads (`PropagatingEffect`, `AlternatableValue`) for safety logic |
| `deep_causality_multivector` | Geometric Algebra for kinematics and relativistic physics |
| `deep_causality_tensor` | Tensor operations for map-based navigation |
| `deep_causality_calculus` | Arrow calculus: the `Rk4` integration operator (the turbulence-forecast march) |
| `deep_causality_physics` | The shipped relativistic-clock kernel carried through the GPS-denial blackout (`ins_gnss_blackout`) |
| `deep_causality_file` | Real RINEX GNSS (SP3/CLK) ingestion over the haft IO monad (`ins_gnss_blackout`) |

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
