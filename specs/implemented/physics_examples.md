# Physics Examples Specification

This document specifies a set of new examples to be added to `examples/physics_examples/` to demonstrate the capabilities of the `deep_causality_physics` crate, emphasizing the use of **Higher-Kinded Types (HKT)** and **Causal Monads** (`PropagatingEffect` / `CausalEffectPropagationProcess`) for multi-physics simulations.

## 1. Astro Example: `event_horizon_probe`

**Concept**: A space probe falling towards a Black Hole, transitioning from Newtonian mechanics to Relativistic effects.

**Domain**: Astrophysics / General Relativity.

**Goal**: Demonstrate switching between physics kernels based on state context (distance to mass).

**Causal Chain**:
1.  **State Initialization**: Probe mass, velocity, distance from Singularity.
2.  **Gravity Assessment**: Calculate `schwarzschild_radius`.
3.  **Regime Check**:
    *   If $r \gg r_s$: Use `orbital_velocity` (Newtonian).
    *   If $r \approx r_s$: Use `spacetime_interval` and `time_dilation_angle` to model relativistic effects.
4.  **Escape Check**: Compare current velocity vs `escape_velocity`.
5.  **Outcome**: Determine if probe escapes, orbits, or crosses the horizon.

**Key APIs**:
*   `deep_causality_physics::astro::schwarzschild_radius`
*   `deep_causality_physics::astro::orbital_velocity`
*   `deep_causality_physics::astro::escape_velocity`
*   `deep_causality_physics::relativity::time_dilation_angle`

---

## 2. Condensed Matter Example: `ginzburg_landau_phase_transition`

**Concept**: State-dependent evolution of a superconducting order parameter, inspired by the "state-dependent causality" analysis in *Martínez-Sánchez & Lozano-Durán (2025)*.

**Domain**: Condensed Matter Physics (Phase Transitions).

**Goal**: Simulate a system where the *causal driver* of the dynamics changes based on the system state (Temperature), mirroring the source-dependent benchmark cases in the referenced paper.

**Physical Model**:
*   **System**: A material described by an Order Parameter $\psi$ and Temperature $T$.
*   **State-Dependent Dynamics**:
    *   **Regime A (Normal, $T > T_c$)**: Dynamics driven by entropic decay (Noise/Diffusion). The "Unique Causality" flows from thermal fluctuations.
    *   **Regime B (Superconducting, $T < T_c$)**: Dynamics driven by the Ginzburg-Landau potential (Coherent Ordering). The "Unique Causality" flows from the order parameter self-interaction.
*   **Implementation**: Use the `ginzburg_landau_free_energy` wrapper to compute the energy landscape $F$ at each step. The state update $\Delta \psi \propto -\nabla F$ will naturally shift behavior as parameters $\alpha(T)$ change sign.

**Causal Chain**:
1.  **Environment**: Update Temperature $T(t)$ (e.g., cooling ramp).
2.  **Parameter Update**: Update $\alpha(T) \propto (T - T_c)$.
3.  **Energy Calculation**: Compute `ginzburg_landau_free_energy`.
4.  **State Evolution**: Update $\psi$ based on energy gradient.
5.  **Causal Analysis (Mock)**: Log which term (Entropy vs Potential) dominates the update, demonstrating state-dependent causal regimes.

**Key APIs**:
*   `deep_causality_physics::condensed::ginzburg_landau_free_energy`
*   `deep_causality_physics::condensed::OrderParameter`

---

## 3. Fluids Example: `bernoulli_flow_network`

**Concept**: Simulation of a pipe network with varying elevation and diameter, solving for pressure distribution.

**Domain**: Fluid Dynamics.

**Goal**: Demonstrate chaining of fluid states through a graph-like structure using the Causal Monad.

**Causal Chain**:
1.  **Source**: Reservoir at height $h_0$ with pressure $P_0$.
2.  **Segment 1 (Pipe)**: Flow to $h_1$, diameter $d_1$. Calculate velocity $v_1$ (mass conservation) -> Calculate $P_1$ via `bernoulli_pressure`.
3.  **Segment 2 (Venturi)**: Constriction to $d_2$. Calculate $v_2$ -> Calculate $P_2$.
4.  **Segment 3 (Vertical)**: Drop to $h_3$. Calculate hydrostatic component via `hydrostatic_pressure`.
5.  **Output**: Final pressure at outlet.

**Key APIs**:
*   `deep_causality_physics::fluids::bernoulli_pressure`
*   `deep_causality_physics::fluids::hydrostatic_pressure`
*   `deep_causality_physics::fluids::{Pressure, Speed, Density}`

---

## 4. Photonics Example: `laser_resonator_stability`

**Concept**: Analyzing the stability of an optical cavity using ABCD matrix propagation.

**Domain**: Photonics / Optics.

**Goal**: Propagate a Gaussian Beam parameter $q$ through multiple optical elements (Free space -> Lens -> Free space -> Mirror) and check for confinement.

**Causal Chain**:
1.  **Beam Init**: Initial Complex Beam Parameter $q_{in}$ (waist, curvature).
2.  **Element 1**: Propagate through Free Space ($M_{free}$). Use `gaussian_q_propagation`.
3.  **Element 2**: Propagate through Thermal Lens ($M_{lens}$). Use `lens_maker` to get $f$, then construct $M$.
4.  **Element 3**: Reflection (Mirror).
5.  **Stability Check**: Verify if the beam spot size remains finite ($w(z)$ via `beam_spot_size`).

**Key APIs**:
*   `deep_causality_physics::photonics::gaussian_q_propagation`
*   `deep_causality_physics::photonics::beam_spot_size`
*   `deep_causality_physics::photonics::lens_maker`
*   `deep_causality_physics::photonics::{AbcdMatrix, ComplexBeamParameter}`

---

## 5. Thermodynamics Example: `carnot_cycle_engine`

**Concept**: Discrete simulation of a 4-stage Carnot Heat Engine cycle.

**Domain**: Thermodynamics.

**Goal**: Track the state $(P, V, T, S)$ of a working gas through Isothermal and Adiabatic processes.

**Causal Chain**:
1.  **State 1 (Start)**: Initial $P_1, V_1, T_H$.
2.  **Expansion (Isothermal)**: Heat input $Q_{in}$. Calculate $\Delta S$ via `shannon_entropy` (conceptually) or `entropy` change. Update to State 2 using `ideal_gas_law`.
3.  **Expansion (Adiabatic)**: Cool to $T_C$. Update to State 3.
4.  **Compression (Isothermal)**: Heat output $Q_{out}$. Update to State 4.
5.  **Compression (Adiabatic)**: Warm to $T_H$. Return to State 1.
6.  **Efficiency Check**: Calculate work done and compare with `carnot_efficiency`.

**Key APIs**:
*   `deep_causality_physics::thermodynamics::ideal_gas_law`
*   `deep_causality_physics::thermodynamics::carnot_efficiency`
*   `deep_causality_physics::thermodynamics::heat_capacity`
