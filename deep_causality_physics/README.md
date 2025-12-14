# DeepCausality Physics

**A library of physics formulas and engineering primitives for DeepCausality.**

`deep_causality_physics` provides a comprehensive collection of physics kernels, causal wrappers, and physical quantities designed for use within the DeepCausality hyper-graph simulation engine. It leverages Geometric Algebra (via `deep_causality_multivector`) and Causal Tensors to model complex physical interactions with high fidelity.

## 🚀 Features

This crate is organized into modular domains, each providing low-level computation kernels and high-level causal wrappers:

*   **🌌 Astro**: Astrophysics kernels (Schwarzschild radius, orbital velocity, luminosity, Hubble's law, etc.).
*   **⚛️ Condensed**: Condensed matter physics (Quantum Geometry, Twistronics, Phase Field Models, e.g., Quantum Geometric Tensor, Bistritzer-MacDonald Hamiltonian, Ginzburg-Landau equations).
*   **📐 Dynamics**: Classical mechanics (Kinematics, Newton's laws), state estimation (Kalman filters), and Euler integration.
*   **⚡ Electromagnetism**: Maxwell's equations, Lorentz force, Poynting vectors, and gauge fields using Geometric Algebra.
*   **💧 Fluids**: Fluid dynamics (Bernoulli's principle, Reynolds number, viscosity, pressure).
*   **🧱 Materials**: Material science properties (Stress, Strain, Hooke's Law, Young's modulus, thermal expansion).
*   **🧲 MHD**: Magnetohydrodynamics (Alfven waves, Magnetic Pressure, Ideal Induction on Manifolds, General Relativistic MHD, Plasma parameters).
*   **☢️ Nuclear**: Nuclear physics (Binding energy, radioactive decay, half-life calculations).
*   **💡 Photonics**: Ray Optics, Polarization Calculus, Gaussian Beam Optics, and Diffraction.
*   **⚛️ Quantum**: Quantum mechanics primitives (Wavefunctions, operators, gates, expectation values, Haruna's Gauge Field gates).
*   **🕰️ Relativity**: Special and General Relativity (Spacetime intervals, time dilation, Einstein tensor, geodesic deviation).
*   **🔥 Thermodynamics**: Statistical mechanics (Entropy, Carnot efficiency, Ideal Gas Law, heat diffusion).
*   **📏 Units**: Type-safe physical units (Time, Mass, Length, ElectricCurrent, Temperature, etc.) to prevent dimensional errors.
*   **🌊 Waves**: Wave mechanics (Doppler effect, wave speed, frequency/wavelength relations).

## 🏗️ Architecture

The library follows a functional and causal architecture:

1.  **Kernels (`*::mechanics`, `*::gravity`, etc.)**: Pure functions that perform the raw physical computations. They operate on `CausalTensor`, `CausalMultiVector`, or primitive types. They are stateless and side-effect free.
    *   *Example*: `klein_gordon_kernel` computes $(\Delta + m^2)\psi$.

2.  **Wrappers (`*::wrappers`)**: Monadic wrappers that lift kernels into the `PropagatingEffect` monad. These allow physics functions to be directly embedded into `CausalEffect` functions within a DeepCausality graph.
    *   *Example*: `apply_gate` wraps `apply_gate_kernel` to validly propagate state changes in the causal graph.

3.  **Quantities (`*::quantities`, `units::*`)**: Newtype wrappers (e.g., `Speed`, `Mass`, `Temperature`) that enforce physical invariants (e.g., mass cannot be negative) and type safety.

## 📦 Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_physics = { version = "0.1.0" }
```

### Example: Relativistic Dynamics

```rust
use deep_causality_physics::{
    time_dilation_angle, spacetime_interval,
    Speed, Time, Length
};
use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define Spacetime Events using Geometric Algebra
    // Minkowski Metric (+---) for 4D spacetime
    let metric = Metric::Minkowski(4);
    
    // Event A at origin
    let event_a = CausalMultiVector::new(vec![0.0; 16], metric).unwrap();
    
    // Event B at (t=10, x=5, y=0, z=0)
    // Multivector data layout depends on dimension, assume standard layout
    let mut data_b = vec![0.0; 16];
    data_b[1] = 10.0; // Time basis
    data_b[2] = 5.0;  // X basis
    let event_b = CausalMultiVector::new(data_b, metric).unwrap();
    
    // 2. Compute Spacetime Interval (Wrapper returns PropagatingEffect)
    let interval_effect = spacetime_interval(&event_b, &metric);
    
    if let Ok(s2) = interval_effect {
        println!("Spacetime Interval s^2: {}", s2);
    }
    
    Ok(())
}
```

Code examples are in the [repo example folder](../examples/physics_examples). 

## 🛠️ Configuration

The crate supports `no_std` environments via feature flags.

*   `default`: Enables `std`.
*   `std`: Usage of standard library (includes `alloc`).
*   `alloc`: Usage of allocation (Vec, String) without full `std`.

## 📜 License

Licensed under MIT. Copyright (c) 2025 DeepCausality Authors.
