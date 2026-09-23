# Event Horizon Probe

This example simulates a probe falling towards a black hole and switches between Newtonian and relativistic physics as the probe nears the event horizon.

## How to Run

```bash
cargo run -p physics_examples --example event_horizon_probe
```

---

## Physics Overview

The probe falls towards a supermassive black hole of Sgr A*'s mass, halving its distance each step.

1.  **Far Field ($r \gg R_s$)**: Newtonian mechanics applies. Gravity is a simple force, and the escape velocity is $\sqrt{2GM/r}$.
2.  **Near Field ($r \approx R_s$)**: Near the Schwarzschild radius ($R_s = 2GM/c^2$) the simulation switches to relativistic physics and computes the **Rapidity** ($\eta$) and time dilation with Geometric Algebra on Minkowski spacetime.

## Key Concepts

*   **Regime Switching**: Each step runs as a stateful `CausalFlow` that carries the probe state and the black-hole mass and picks the physics from the distance.
*   **Automatic Differentiation**: The gravitational acceleration and tidal gradient are the first and second derivatives of the potential $\Phi(r) = -GM/r$.
*   **Geometric Algebra**: Computes the relativistic rapidity without coordinates.
*   **Causal Chain**: State -> Distance Check -> Physics Kernel Selection -> State Update.

## APIs Used

*   `schwarzschild_radius`
*   `escape_velocity`
*   `time_dilation_angle` (Relativity)
