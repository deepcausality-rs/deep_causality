# Physics Kernels

This document covers the **kernel layer** of `deep_causality_physics`: pure, stateless,
domain-specific computations and the typed quantity wrappers around them.

For the **theory layer** (Gauge Theories on a shared topological backend) see
[README_GAUGE_THEORIES.md](./README_GAUGE_THEORIES.md).
For the project overview see [README.md](./README.md).

---

## Domains

Kernels live under `src/kernels/<domain>/`. Each domain holds pure computation kernels and monadic
wrappers (`wrappers.rs`); its quantity newtypes live under `src/quantities/<domain>/`. The table
lists each domain's kernel source files (excluding `mod.rs` and `wrappers.rs`) and what they cover.

| Domain | Source files | Coverage |
|---|---|---|
| **Astro** | `ks_constraint`, `ks_propagator`, `mechanics`, `two_body` | Schwarzschild radius, orbital velocity, escape velocity, gravitational redshift; exact two-body (Kepler) propagation, Kustaanheimo–Stiefel regularised propagation and constraint projection. |
| **Chronometric** | `forward_clock`, `solve_gm`, `wrapper` | Forward proper-time rate and clock-offset kernels. Inverts the J2-corrected weak-field 1PN clock equation (Bjerhammar 1975, Vermeer 1983) to recover $GM_\oplus$ and the derived planetary mass from satellite clock time-dilation measurements. `CentralBody` (gravity-model parameters), `SpaceTimeCoordinate` (clock + kinematic state), `solve_gm_analytical`. |
| **Condensed** | `moire`, `phase`, `qgt` | Twistronics (Bistritzer–MacDonald moiré Hamiltonian), phase-field models (Ginzburg–Landau, Cahn–Hilliard), Quantum Geometric Tensor. |
| **Dynamics** | `estimation`, `kinematics` | Classical mechanics (Newton's laws, kinematics), state estimation (Kalman filters), Euler integration. |
| **Electromagnetism (`em`)** | `fields`, `forces`, `solver` | Maxwell's equations, Lorentz force, Poynting vectors, gauge fields via Geometric Algebra. |
| **Hypersonic** | `finite_rate`, `ionization`, `shock`, `thermochemistry` | Park two-temperature reacting air: vibrational relaxation, Arrhenius rates, Saha / Park-2T ionization, Rankine–Hugoniot temperature jump, recovery-temperature reconstruction. |
| **Fluids** | `boundary_layer`, `coherent_structures`, `compressible`, `constitutive`, `dimensionless`, `governing`, `ideal_flow`, `kinematics`, `mechanics`, `turbulence` | Full Navier–Stokes surface: continuity / momentum / energy RHS, Newtonian and power-law stress, kinematics (S, Ω, vorticity, CPC invariants), 18 dimensionless numbers, turbulence quantities (TKE, ε, Kolmogorov scales, Reynolds stress, Boussinesq), coherent-structure detectors (Q, Δ, λ₂, swirling strength), compressible thermodynamics (speed of sound, isentropic stagnation, entropy production), wall functions, ideal-flow primitives (Bernoulli, stream function, circulation, Kutta–Joukowski). |
| **Materials** | `mechanics` | Stress, strain, Hooke's law, Young's modulus, thermal expansion. |
| **MHD** | `grmhd`, `ideal`, `plasma`, `resistive` | Alfvén waves, magnetic pressure, ideal induction on manifolds, General Relativistic MHD with manifold-based current computation, plasma parameters. |
| **Nuclear** | `lund`, `pdg`, `physics`, `qcd` | Binding energy, radioactive decay, PDG particle database, **Lund String Fragmentation** for QCD hadronization. |
| **Propulsion** | `descent`, `nozzle`, `performance`, `plume`, `srp` | Rocket performance, nozzle exit state, supersonic-retropropulsion similarity numbers and Jarvinen–Adams drag correlation, plume-boundary geometry, powered-descent closed forms. |
| **Photonics** | `beam`, `diffraction`, `polarization`, `ray` | Ray optics, polarization calculus, Gaussian beam optics, diffraction. |
| **Quantum** | `mechanics` | Klein–Gordon operator on a simplicial manifold. Gates, Haruna gates, and Dirac-notation operations live in `deep_causality_quantum`. |
| **Relativity** | `gravity`, `spacetime` | Special and General Relativity: spacetime intervals, time dilation, Einstein tensor, geodesic deviation. |
| **Thermodynamics** | `stats` | Statistical mechanics: entropy, Carnot efficiency, ideal gas law, heat diffusion. |
| **Waves** | `general` | Doppler effect, wave speed, frequency/wavelength relations. |

---

## Architecture

Kernels are organized in four layers:

1. **Kernels (`kernels/<domain>/mechanics.rs`, `kernels/<domain>/gravity.rs`, etc.)** — Pure,
   stateless functions that compute the physics. They operate on `CausalTensor`,
   `CausalMultiVector`, `Manifold`, or primitive types.
   *Example*: `klein_gordon_kernel` computes $(\Delta + m^2)\psi$.

2. **Wrappers (`kernels/<domain>/wrappers.rs`)** — Monadic wrappers that lift kernels into the
   `PropagatingEffect` monad, so a causal function in a DeepCausality graph can call a physics
   function directly.
   *Example*: `klein_gordon` wraps `klein_gordon_kernel` and returns its result or error as a
   `PropagatingEffect`.

3. **Quantities (`quantities/<domain>/`)** — Newtype wrappers
   (`Speed`, `Mass`, `Temperature`, `FourMomentum`, `Hadron`, …) that enforce physical invariants
   (e.g. mass cannot be negative). **Every wrapper is generic over
   `R: RealField`**, so the same code runs at `f32`, `f64`, `Float106`, or any other field that
   implements `RealField`.

4. **Metric Types** — Re-exports from `deep_causality_metric` for sign-convention handling
   (`LorentzianMetric`, `EastCoastMetric`, `WestCoastMetric`, `RelativityMetric`, `ParticleMetric`).

---

## When to use kernels vs. theories

* **Kernels for Isolation**: To solve a single equation (e.g., `schwarzschild_radius`,
  `lorentz_force`, `cahn_hilliard_flux`), use the standalone kernels.
* **Theories for Frameworks**: To work within a full physical theory (General Relativity,
  Electroweak Theory, etc.), use the theory modules under `src/theories/`. See
  [README_GAUGE_THEORIES.md](./README_GAUGE_THEORIES.md).

---

## Examples

### Relativistic Dynamics

```rust
use deep_causality_physics::{spacetime_interval, time_dilation_angle};
use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define Spacetime Events using Geometric Algebra
    // Minkowski Metric (+---) for 4D spacetime
    let metric = Metric::Minkowski(4);

    // Event A at origin
    let event_a = CausalMultiVector::<f64>::new(vec![0.0; 16], metric).unwrap();

    // Event B at (t=10, x=5, y=0, z=0)
    let mut data_b = vec![0.0; 16];
    data_b[1] = 10.0; // Time basis
    data_b[2] = 5.0;  // X basis
    let event_b = CausalMultiVector::<f64>::new(data_b, metric).unwrap();

    // 2. Compute Spacetime Interval (Wrapper returns PropagatingEffect)
    let interval_effect = spacetime_interval(&event_b, &metric);

    if let Ok(s2) = interval_effect.value().clone().into_value() {
        println!("Spacetime Interval s^2: {}", s2);
    }

    Ok(())
}
```

### Chronometric GM Recovery

```rust
use deep_causality_physics::{CentralBody, SpaceTimeCoordinate, solve_gm_analytical};

fn main() {
    // Two space-time coordinate samples (position in ECI, inertial velocity,
    // IGS clock bias and drift rate). In practice these come from preprocessed
    // satellite broadcast clock + SP3 orbit data.
    let coord_a: SpaceTimeCoordinate<f64> = /* ... */;
    let coord_b: SpaceTimeCoordinate<f64> = /* ... */;

    // Earth parameters: JGM-3 J2 with WGS-84 equatorial radius. The struct
    // pairs J2 with its reference radius — they are only meaningful together.
    let body = CentralBody::EARTH_JGM3;

    // Wrapper returns PropagatingEffect<f64> for use in causaloid graphs.
    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    // → recovered GM in m³/s²; divide by NEWTONIAN_CONSTANT_OF_GRAVITATION
    //   to get Earth's mass in kg.
}
```

[`examples/physics_examples/chronometric_gm_recovery`](../examples/physics_examples/chronometric_gm_recovery)
processes one full GPS week of Galileo broadcast clock data (satellite E14). It runs the
`CausalMonad` bind chain end to end and recovers $GM_\oplus$ and Earth's mass to ~0.2% relative
error against published JGM-3 / IERS 2010 references, from clock time dilation alone.

### Lund String Fragmentation (QCD Hadronization)

```rust
use deep_causality_physics::{FourMomentum, LundParameters, lund_string_fragmentation_kernel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Lund parameters (defaults tuned to LEP e+e- data)
    let params = LundParameters::default();

    // Define a quark-antiquark string (e.g., from e+e- → qq̄)
    // Quark moving in +z, antiquark in -z
    let quark = FourMomentum::<f64>::new(50.0, 0.0, 0.0, 50.0);
    let antiquark = FourMomentum::<f64>::new(50.0, 0.0, 0.0, -50.0);
    let endpoints = vec![(quark, antiquark)];

    // Fragment the string into hadrons
    let mut rng = deep_causality_rand::rng();
    let hadrons = lund_string_fragmentation_kernel(&endpoints, &params, &mut rng)?;

    println!("Produced {} hadrons:", hadrons.len());
    for h in &hadrons {
        println!("  PDG ID: {}, Energy: {:.2} GeV", h.pdg_id(), h.energy());
    }

    Ok(())
}
```

More example code is in the [examples folder](../examples/physics_examples).

---

## Precision

Every kernel and every quantity wrapper in this crate is generic over `R: RealField`. Pick the
precision at the call site:

```rust
use deep_causality_physics::Mass;
use deep_causality_num::Float106;

let m_fast: Mass<f32>           = Mass::new(5.0_f32).unwrap();   // games / viz
let m_std:  Mass<f64>           = Mass::new(5.0_f64).unwrap();   // engineering default
let m_hi:   Mass<Float106>      = Mass::new(Float106::from(5.0)).unwrap(); // cosmology
```

The exception is the physical constants under `src/constants/` and the PDG quark-mass constants
in `kernels/nuclear/pdg.rs`. They are declared as `pub const X: f64 = literal`, and the consumer
converts at the call site via `R::from_f64(SPEED_OF_LIGHT)`. The values gain nothing from a wider
type: exactly defined CODATA constants fit in `f64`, and measured constants carry measurement
uncertainty far larger than `f64` rounding.
