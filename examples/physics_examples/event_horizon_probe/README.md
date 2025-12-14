# Event Horizon Probe

This example simulates a space probe falling towards a black hole, dynamically switching between Newtonian and Relativistic physics models based on its proximity to the event horizon.

## How to Run

```bash
cargo run -p physics_examples --example event_horizon_probe
```

---

## Physics Overview

The simulation tracks a probe falling towards a supermassive black hole (like Sgr A*).

1.  **Far Field ($r \gg R_s$)**: The system uses Newtonian mechanics. Gravity is a simple force, and escape velocity is $\sqrt{2GM/r}$.
2.  **Near Field ($r \approx R_s$)**: As the probe approaches the Schwarzschild radius ($R_s = 2GM/c^2$), the system switches to Relativistic physics. It calculates the **Rapidity** ($\eta$) and Time Dilation using Geometric Algebra on Minkowski spacetime.

## Key Concepts

*   **Regime Switching**: The Causal Monad (`PropagatingEffect`) allows logic that adapts to the state context.
*   **Geometric Algebra**: Used to calculate relativistic rapidity in a coordinate-free manner.
*   **Causal Chain**: State -> Distance Check -> Physics Kernel Selection -> State Update.

## APIs Used

*   `schwarzschild_radius`
*   `escape_velocity`
*   `time_dilation_angle` (Relativity)
