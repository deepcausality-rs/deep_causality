# Physics Kernels

This document covers the **kernel layer** of `deep_causality_physics` — the pure, stateless,
domain-specific computations and the type-safe quantity wrappers built around them.

For the **theory layer** (Gauge Theories on a shared topological backend) see
[README_GAUGE_THEORIES.md](./README_GAUGE_THEORIES.md).
For the project overview see [README.md](./README.md).

---

## Domains

Kernels live under `src/kernels/<domain>/`. Each domain ships pure computation kernels, monadic
wrappers, and domain-specific quantity wrappers.

* **Astro**: Astrophysics kernels (Schwarzschild radius, orbital velocity, escape velocity, etc.).
* **Chronometric**: Chronometric geodesy kernels — invert the J2-corrected weak-field 1PN clock
  equation (Bjerhammar 1975, Vermeer 1983) to recover gravitational parameters such as $GM_\oplus$
  and the derived planetary mass $M_\oplus = GM_\oplus / G$ from satellite clock time-dilation
  measurements. Includes `CentralBody` (gravity-model parameters), `SpaceTimeCoordinate` (clock +
  kinematic state), and the `solve_gm_analytical` kernel and wrapper.
* **Condensed**: Condensed matter physics (Quantum Geometry, Twistronics, Phase Field Models, e.g.,
  Quantum Geometric Tensor, Bistritzer-MacDonald Hamiltonian, Ginzburg-Landau equations).
* **Dynamics**: Classical mechanics (Kinematics, Newton's laws), state estimation (Kalman filters),
  and Euler integration.
* **Electromagnetism (`em`)**: Maxwell's equations, Lorentz force, Poynting vectors, and gauge
  fields using Geometric Algebra.
* **Fluids**: Fluid dynamics (Bernoulli's principle, Reynolds number, viscosity, pressure).
* **Materials**: Material science properties (Stress, Strain, Hooke's Law, Young's modulus, thermal
  expansion).
* **MHD**: Magnetohydrodynamics (Alfven waves, Magnetic Pressure, Ideal Induction on Manifolds,
  General Relativistic MHD with Manifold-based current computation, Plasma parameters).
* **Nuclear**: Nuclear physics (Binding energy, radioactive decay, PDG particle database,
  **Lund String Fragmentation** for QCD hadronization).
* **Photonics**: Ray Optics, Polarization Calculus, Gaussian Beam Optics, and Diffraction.
* **Quantum**: Quantum mechanics primitives (Wavefunctions, operators, gates, expectation values,
  Haruna's Gauge Field gates).
* **Relativity**: Special and General Relativity (Spacetime intervals, time dilation, Einstein
  tensor, geodesic deviation).
* **Thermodynamics**: Statistical mechanics (Entropy, Carnot efficiency, Ideal Gas Law, heat
  diffusion).
* **Waves**: Wave mechanics (Doppler effect, wave speed, frequency/wavelength relations).

---

## Architecture

Kernels are organized in four layers:

1. **Kernels (`kernels/<domain>/mechanics.rs`, `kernels/<domain>/gravity.rs`, etc.)** — Pure
   functions that perform the raw physical computations. They operate on `CausalTensor`,
   `CausalMultiVector`, `Manifold`, or primitive types. Stateless, side-effect free.
   *Example*: `klein_gordon_kernel` computes $(\Delta + m^2)\psi$.

2. **Wrappers (`kernels/<domain>/wrappers.rs`)** — Monadic wrappers that lift kernels into the
   `PropagatingEffect` monad. They allow physics functions to be directly embedded into
   `CausalEffect` functions within a DeepCausality graph.
   *Example*: `apply_gate` wraps `apply_gate_kernel` to validly propagate state changes in the
   causal graph.

3. **Quantities (`kernels/<domain>/quantities.rs`, `units::*`)** — Newtype wrappers
   (`Speed`, `Mass`, `Temperature`, `FourMomentum`, `Hadron`, …) that enforce physical invariants
   (e.g. mass cannot be negative) and type safety. **Every wrapper is generic over
   `R: RealField`**, so the same code runs at `f32`, `f64`, `DoubleFloat`, or any other field that
   implements `RealField`.

4. **Metric Types** — Re-exports from `deep_causality_metric` for sign-convention handling
   (`LorentzianMetric`, `EastCoastMetric`, `WestCoastMetric`, `PhysicsMetric`).

---

## When to use kernels vs. theories

* **Kernels for Isolation**: If you need to solve a specific equation (e.g., `schwarzschild_radius`,
  `lorentz_force`, `cahn_hilliard_flux`) in isolation, use the standalone kernels. They are pure
  functions, stateless, and efficient.
* **Theories for Frameworks**: If you are working within a full physical theory (General Relativity,
  Electroweak Theory, etc.), use the Physics Theories modules under `src/theories/`. See
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

A complete worked example processing one full GPS week of Galileo broadcast clock data
(satellite E14) is in [`examples/chronometric_examples/gm_recovery`](../examples/chronometric_examples/gm_recovery).
It demonstrates the framework's `CausalMonad` bind chain end-to-end and recovers
$GM_\oplus$ and Earth's mass to ~0.2% relative error against published JGM-3 / IERS 2010
references — *the planet weighed by clock time-dilation alone.*

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
use deep_causality_num::DoubleFloat;

let m_fast: Mass<f32>           = Mass::new(5.0_f32).unwrap();   // games / viz
let m_std:  Mass<f64>           = Mass::new(5.0_f64).unwrap();   // engineering default
let m_hi:   Mass<DoubleFloat>   = Mass::new(DoubleFloat::from(5.0)).unwrap(); // cosmology
```

The single carve-out is physical constants under `src/constants/` and the PDG quark-mass constants
in `kernels/nuclear/pdg.rs`. Those stay declared as `pub const X: f64 = literal` and the consumer
converts at the call site via `R::from_f64(SPEED_OF_LIGHT)`. Rationale: the values themselves do not
benefit from precision-parametricity (exact-defined CODATA constants fit in `f64` exactly; measured
constants have measurement uncertainty far below `f64` precision).
