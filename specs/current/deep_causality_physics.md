# Deep Causality Physics Specification

**Crate Name:** `deep_causality_physics`
**Version:** 0.1.0 (Draft)
**Status:** Specification / Planning

## 1. Overview

The `deep_causality_physics` crate provides a standard library of physics formulas and engineering primitives, fully integrated with the `deep_causality` ecosystem.

**Key Definition:** All public physics functions in this crate MUST return `PropagatingEffect<T>`.
*   **Why?** This ensures every calculation—from a Quantum Gate to a Reynolds number check—is traceable (via `EffectLog`), has standardized error handling (`CausalityError`), and composes natively within `CausalMonad` chains.

## 2. Dependencies

The crate will depend on the following internal crates:
*   `deep_causality_core`: For `PropagatingEffect`, `CausalityError`.
*   `deep_causality_multivector`: For `MultiVector` (Geometric Algebra), `Metric`.
*   `deep_causality_tensor`: For `CausalTensor` (Tensor Algebra).
*   `deep_causality_topology`: For `Topology`, `Manifold` (Differential Geometry).
*   `deep_causality_num`: For numerical traits (`Real`, `Complex`, `DixonAlgebra`).

## 3. Directory Structure

The crate will follow the `AGENTS.md` conventions:
*   **Source:** `src/` (One type/module per folder/file).
*   **Tests:** `tests/` (Replicates `src/` structure).

```text
deep_causality_physics/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Exports all submodules
│   ├── error/
│   │   ├── mod.rs
│   │   └── physics_error.rs # PhysicsError Enum
│   ├── constants/          # CODATA 2022 Physical Constants
│   │   ├── mod.rs
│   │   ├── universal.rs
│   │   ├── electromagnetic.rs
│   │   ├── atomic.rs
│   │   └── thermodynamics.rs
│   ├── quantum/
│   │   ├── mod.rs          
│   │   ├── quantities.rs   # Probability, Energy, Time, PhaseAngle
│   │   ├── gates.rs        
│   │   └── mechanics.rs    
│   ├── electromagnetism/
│   │   ├── mod.rs
│   │   ├── quantities.rs   # ElectricPotential, MagneticFlux
│   │   ├── forces.rs       
│   │   └── fields.rs       
│   ├── relativity/
│   │   ├── mod.rs
│   │   ├── quantities.rs   # SpacetimeInterval
│   │   ├── spacetime.rs    
│   │   └── gravity.rs      
│   ├── dynamics/
│   │   ├── mod.rs
│   │   ├── quantities.rs   # Mass, Speed, Acceleration, Force, Torque
│   │   └── kinematics.rs   
│   ├── thermodynamics/     
│   │   ├── mod.rs
│   │   ├── quantities.rs   # Temperature, Entropy
│   │   └── stats.rs
│   ├── materials/          
│   │   ├── mod.rs
│   │   ├── quantities.rs   # Stress, Stiffness
│   │   └── mechanics.rs
│   ├── fluids/             
│   │   ├── mod.rs
│   │   ├── quantities.rs   # Density, Viscosity, Pressure
│   │   └── aerodynamics.rs
│   ├── nuclear/            
│   │   ├── mod.rs
│   │   ├── quantities.rs   # HalfLife, Activity, AmountOfSubstance
│   │   └── decay.rs
│   ├── waves/              
│   │   ├── mod.rs
│   │   ├── quantities.rs   # Frequency, Wavelength
│   │   └── optics.rs
│   └── astro/              
│       ├── mod.rs
│       ├── quantities.rs   # OrbitalParam
│       └── orbital.rs
└── tests/
    ├── quantum/
    │   ├── mechanics_tests.rs
    │   └── gates_tests.rs
    ├── electromagnetism/
    │   ├── forces_tests.rs
    │   └── fields_tests.rs
    ├── ... (matching src structure)
```

## 4. Domain Types (Newtype Pattern)

To ensure type safety, all physical quantities MUST be wrapped in "Newtype" structs with **private fields** to enforce invariants. These are defined in the `quantities.rs` file within each respective submodule.

**Standard Pattern for All Quantities:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct Mass(f64); // Private field

impl Mass {
    /// Creates a new instance with validation.
    pub fn new(val: f64) -> Result<Self, CausalityError> {
        if val < 0.0 { return Err(CausalityError::PhysicalInvariantBroken("Negative Mass")); }
        Ok(Self(val))
    }

    /// Creates a new instance bypassing validation (Performance Critical).
    /// Safe to call, but caller ensures validity. NO UNSAFE BLOCK.
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }

    pub fn value(&self) -> f64 { self.0 }
}

impl From<Mass> for f64 { fn from(val: Mass) -> Self { val.0 } }
```

**List of Domain Types (Follows Standard Pattern):**

```rust
// src/quantum/quantities.rs
pub struct Probability(f64);
pub struct Energy(f64);
pub struct Time(f64);
pub struct PhaseAngle(f64);

// src/electromagnetism/quantities.rs
pub struct ElectricPotential(f64);
pub struct MagneticFlux(f64);

// src/dynamics/quantities.rs
pub struct Mass(f64);
pub struct Speed(f64);
pub struct Acceleration(f64);
pub struct Length(f64);
pub struct Area(f64);
pub struct Volume(f64);
pub struct Force(f64);
pub struct Torque(f64);
pub struct AngularMomentum(f64);
pub struct MomentOfInertia(f64);
pub struct Frequency(f64);

// src/fluids/quantities.rs & src/materials/quantities.rs
pub struct Density(f64);
pub struct Viscosity(f64);
pub struct Pressure(f64);
pub struct Temperature(f64);

// src/nuclear/quantities.rs
pub struct AmountOfSubstance(f64); // Moles
pub struct HalfLife(f64);
pub struct Activity(f64); // Becquerels

// src/lib.rs (Global dimensionless ratios)
pub struct Ratio(f64);
pub struct IndexOfRefraction(f64);
pub struct Efficiency(f64);
```

## 4.1. Physical Constants (CODATA 2022)

Constants are organized by domain in the `src/constants/` directory. All are `pub const f64`.

```rust
// src/constants/universal.rs
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0; // m s^-1 (exact)
pub const PLANCK_CONSTANT: f64 = 6.626_070_15e-34; // J Hz^-1 (exact)
pub const REDUCED_PLANCK_CONSTANT: f64 = 1.054_571_817e-34; // J s
pub const VACUUM_MAGNETIC_PERMEABILITY: f64 = 1.256_637_061_27e-6; // N A^-2
pub const VACUUM_ELECTRIC_PERMITTIVITY: f64 = 8.854_187_818_8e-12; // F m^-1
pub const NEWTONIAN_CONSTANT_OF_GRAVITATION: f64 = 6.674_30e-11; // m^3 kg^-1 s^-2

// src/constants/electromagnetic.rs
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // C (exact)
pub const FINE_STRUCTURE_CONSTANT: f64 = 7.297_352_564_3e-3; // Dimensionless

// src/constants/atomic.rs
pub const ELECTRON_MASS: f64 = 9.109_383_713_9e-31; // kg
pub const PROTON_MASS: f64 = 1.672_621_925_95e-27; // kg
pub const ATOMIC_MASS_CONSTANT: f64 = 1.660_539_068_92e-27; // kg

// src/constants/thermodynamics.rs
pub const BOLTZMANN_CONSTANT: f64 = 1.380_649e-23; // J K^-1 (exact)
pub const STEFAN_BOLTZMANN_CONSTANT: f64 = 5.670_374_419e-8; // W m^-2 K^-4
pub const AVOGADRO_CONSTANT: f64 = 6.022_140_76e23; // mol^-1 (exact)
```

## 4.2. Error Handling (`src/error/physics_error.rs`)

A specialized `PhysicsError` enum captures domain-specific invariants.

```rust
// src/error/physics_error.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct PhysicsError(pub PhysicsErrorEnum);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub enum PhysicsErrorEnum {
    // Fundamental
    PhysicalInvariantBroken(String),
    DimensionMismatch(String),

    // Relativistic
    CausalityViolation(String),
    MetricSingularity(String),

    // Quantum
    NormalizationError(String),

    // Thermodynamics
    ZeroKelvinViolation,
    EntropyViolation(String),

    // Numerical
    Singularity(String),
}

impl core::fmt::Display for PhysicsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PhysicsError {}

impl PhysicsError {
    pub fn new(variant: PhysicsErrorEnum) -> Self {
        Self(variant)
    }
}

// Integration with Generic CausalityError
impl From<PhysicsError> for CausalityError {
    fn from(e: PhysicsError) -> Self {
        CausalityError::new(CausalityErrorEnum::PhysicsError(e.to_string()))
    }
}
```

## 5. Module Specifications

**Dual-Layer Architecture:**
To ensure high performance (SIMD/Cache Locality) and Causal Traceability, every module implements a Dual-Layer API:
1.  **Kernels (Pure):** `public fn {name}_kernel(...) -> T`. Pure math, `no_std`, stack-only.
2.  **Causal Wrappers:** `public fn {name}(...) -> PropagatingEffect<T>`. Monadic wrapper around the kernel.

*Below lists primarily the Causal Wrappers, but the implementation MUST include the corresponding Kernels.*

### 5.1. Quantum (`quantum`)

*   **Logic:** Hilbert Space operations.
*   **Dependencies:** `deep_causality_multivector` (HilbertState).
*   **Consolidation:** Moves `QuantumGates` and `QuantumOps` from `deep_causality_multivector`.

```rust
// Core Operations (Moved from Multivector Extension)
pub trait QuantumGates {
    fn gate_identity() -> Self;
    fn gate_x() -> Self;
    fn gate_y() -> Self;
    fn gate_z() -> Self;
    fn gate_hadamard() -> Self;
    fn gate_cnot() -> Self; // If applicable to single MV or Tensor
}

pub trait QuantumOps {
    fn dag(&self) -> Self; // Hermitian Conjugate
    fn bracket(&self, other: &Self) -> Complex<f64>; // Inner Product
    fn expectation_value(&self, operator: &Self) -> Complex<f64>;
    fn normalize(&self) -> Self;
}

// Physics API (Monadic Wrappers)
pub fn born_probability(state: &HilbertState, basis: &HilbertState) -> PropagatingEffect<Probability>;
pub fn expectation_value(state: &HilbertState, operator: &Operator) -> PropagatingEffect<f64>; // Result can be any observable
pub fn apply_gate(state: &HilbertState, gate: &Gate) -> PropagatingEffect<HilbertState>;
pub fn commutator(a: &MultiVector, b: &MultiVector) -> PropagatingEffect<MultiVector>;
pub fn haruna_s_gate(field: &MultiVector) -> PropagatingEffect<Operator>;
pub fn fidelity(ideal: &HilbertState, actual: &HilbertState) -> PropagatingEffect<Probability>;
```

### 5.2. Electromagnetism (`electromagnetism`)

*   **Logic:** Maxwell's equations and plasma forces.
*   **Dependencies:** `deep_causality_multivector`.

```rust
pub fn lorentz_force(j: &MultiVector, b: &MultiVector) -> PropagatingEffect<MultiVector>;
pub fn maxwell_gradient(potential: &MultiVector) -> PropagatingEffect<MultiVector>;
pub fn lorenz_gauge(potential: &MultiVector) -> PropagatingEffect<f64>; // Scalar check
pub fn poynting_vector(e: &MultiVector, b: &MultiVector) -> PropagatingEffect<MultiVector>;
pub fn magnetic_helicity(potential: &MultiVector, field: &MultiVector) -> PropagatingEffect<MagneticFlux>;
```

### 5.3. Relativity (`relativity`)

*   **Logic:** General Relativity and spacetime metrics.
*   **Dependencies:** `deep_causality_tensor`, `deep_causality_multivector`.

```rust
pub fn spacetime_interval(x: &MultiVector, metric: &Metric) -> PropagatingEffect<f64>; // Interval s^2
pub fn einstein_tensor(ricci: &CausalTensor<f64>, scalar_r: f64, metric: &CausalTensor<f64>) -> PropagatingEffect<CausalTensor<f64>>;
pub fn geodesic_deviation(riemann: &CausalTensor<f64>, velocity: &MultiVector) -> PropagatingEffect<MultiVector>;
pub fn time_dilation_angle(t1: &MultiVector, t2: &MultiVector) -> PropagatingEffect<PhaseAngle>;
pub fn chronometric_volume(a: &MultiVector, b: &MultiVector, c: &MultiVector) -> PropagatingEffect<MultiVector>;
```

### 5.4. Dynamics (`dynamics`)

*   **Logic:** Classical kinematics and Rotational dynamics.

```rust
// Linear
pub fn kinetic_energy(mass: Mass, velocity: &MultiVector) -> PropagatingEffect<Energy>;
pub fn phase_space_density(x: &CausalTensor<f64>, v: &CausalTensor<f64>) -> PropagatingEffect<CausalTensor<f64>>;
pub fn transport_cost(p1: &MultiVector, p2: &MultiVector) -> PropagatingEffect<Energy>; // Cost interpreted as Work/Energy
pub fn sprt_confidence(prob_one: Probability, prob_zero: Probability) -> PropagatingEffect<Ratio>;

// Rotational (Major Gap Closed)
pub fn torque(radius: &MultiVector, force: &MultiVector) -> PropagatingEffect<MultiVector>; // Bivector torque
pub fn angular_momentum(radius: &MultiVector, momentum: &MultiVector) -> PropagatingEffect<MultiVector>;
pub fn rotational_kinetic_energy(inertia: MomentOfInertia, omega: Frequency) -> PropagatingEffect<Energy>;
```

---

### 5.5 - 5.10. Engineering Primitives

**Thermodynamics (`thermodynamics`)**
```rust
pub fn boltzmann_factor(energy: Energy, temp: Temperature) -> PropagatingEffect<Probability>;
pub fn shannon_entropy(probs: &CausalTensor<f64>) -> PropagatingEffect<f64>; // Entropy bits/nats
pub fn heat_capacity(diff_energy: Energy, diff_temp: Temperature) -> PropagatingEffect<f64>; // J/K
pub fn partition_function(energies: &CausalTensor<f64>, temp: Temperature) -> PropagatingEffect<f64>;
// Major Standard Additions
pub fn ideal_gas_law(pressure: Pressure, volume: Volume, moles: AmountOfSubstance, temp: Temperature) -> PropagatingEffect<Ratio>; // Returns R (Gas Constant) check or pressure if solving
pub fn carnot_efficiency(temp_hot: Temperature, temp_cold: Temperature) -> PropagatingEffect<Efficiency>;
```

**Materials (`materials`)**
```rust
pub fn hookes_law(stiffness: &CausalTensor<f64>, strain: &CausalTensor<f64>) -> PropagatingEffect<CausalTensor<f64>>;
pub fn von_mises_stress(stress: &CausalTensor<f64>) -> PropagatingEffect<Pressure>;
pub fn thermal_expansion(coeff: f64, delta_temp: Temperature) -> PropagatingEffect<CausalTensor<f64>>;
```

**Fluids (`fluids`)**
```rust
pub fn reynolds_number(density: Density, velocity: Speed, length: Length, viscosity: Viscosity) -> PropagatingEffect<Ratio>; // Dimensionless
pub fn bernoulli_pressure(velocity: Speed, height: Length, density: Density) -> PropagatingEffect<Pressure>;
pub fn mach_number(velocity: &MultiVector, speed_sound: Speed) -> PropagatingEffect<Ratio>;
pub fn vorticity(velocity_field: &CausalMultiVector) -> PropagatingEffect<CausalMultiVector>;
// Major Standard Additions
pub fn drag_equation(density: Density, velocity: Speed, area: Area, coeff_drag: Ratio) -> PropagatingEffect<Force>;
```

**Waves (`waves`)**
```rust
pub fn relativistic_doppler(source_freq: PhaseAngle, relative_beta: Ratio) -> PropagatingEffect<PhaseAngle>; // Frequency/Angle
pub fn phase_velocity(omega: PhaseAngle, k: f64) -> PropagatingEffect<Speed>;
pub fn stokes_parameters(electric_field: &MultiVector) -> PropagatingEffect<MultiVector>;
pub fn snells_law(n1: IndexOfRefraction, n2: IndexOfRefraction, theta1: PhaseAngle) -> PropagatingEffect<PhaseAngle>;
// Major Standard Additions
pub fn rayleigh_criterion(wavelength: Length, diameter: Length) -> PropagatingEffect<PhaseAngle>; // Diffraction limit angle
```

**Nuclear (`nuclear`)**
```rust
// Major Standard Additions (Engineering Safety/Power)
pub fn radioactive_decay(initial_amount: AmountOfSubstance, decay_constant: Ratio, time: Time) -> PropagatingEffect<AmountOfSubstance>;
pub fn half_life_to_decay_constant(half_life: HalfLife) -> PropagatingEffect<Ratio>;
pub fn binding_energy(mass_defect: Mass) -> PropagatingEffect<Energy>; // E=mc^2
```

**Astro (`astro`)**
```rust
pub fn vis_viva(mass: Mass, r: Length, a: Length) -> PropagatingEffect<Speed>;
pub fn escape_velocity(mass: Mass, r: Length) -> PropagatingEffect<Speed>;
pub fn tsiolkovsky(exhaust_vel: Speed, mass_initial: Mass, mass_final: Mass) -> PropagatingEffect<Ratio>; // DeltaV is Speed, but usually returns scalar check
```

## 6. Testing Plan (100% Coverage)

To achieve 100% test coverage, the following strategy is mandatory:

1.  **Unit Tests (`tests/mod_name/file_name_tests.rs`)**
    *   **Positive Cases:** Verify standard inputs produce correct physical results (compare against known constants/textbook examples within floating-point tolerance).
    *   **Negative Cases:** Verify invalid inputs (e.g., negative mass, exceeding speed of light if enforced, mismatched tensor dimensions) return `CausalityError`.
    *   **Boundary Cases:** Zero values, infinity, very small numbers (to check for underflow/precision issues).

2.  **Mocking & Integration**
    *   Since these are generally pure functions wrapped in `PropagatingEffect`, heavy mocking is not required.
    *   However, integration tests should verify that `PropagatingEffect` chains work as expected (e.g., using `.bind()` to chain `lorentz_force` into `dynamics::acceleration`).

3.  **Doc Tests**
    *   Every public function MUST have a distinct documentation example ensuring the code compiles and runs.

## 6. Implementation Guidelines



*   **Safety:** The usage of the `unsafe` keyword is strictly **forbidden**.
    *   `new_unchecked` methods must be implemented safely (e.g., as regular functions that simply skip validation logic, without using `unsafe` blocks).
    *   Zero `unsafe` blocks are allowed in the crate.
*   **Error Handling:** Use `CausalityError` for all logic errors.
*   **Logging:** Use `EffectLog` to record significant physics events (e.g., "Relativistic Reversal detected" in the grmhd example style) *if reasonable*. For pure tight-loop math, logging might be skipped for performance unless an error occurs.
*   **Performance:** Where possible, forward to the optimized implementations in `deep_causality_tensor` and `deep_causality_multivector`.

## 7. Example Migration Plan

This section details the roadmap for replacing manual physics implementations in existing examples with the `deep_causality_physics` standard library.

**Goal:** Clean up example code and demonstrate standardized usage.

### 7.1. Target Directories
*   `deep_causality_multivector/examples/`
*   `deep_causality_topology/examples/`
*   `examples/physics_examples/`

### 7.2. Migration Mapping

| Example File | Legacy Implementation | Target Replacement | Notes |
| :--- | :--- | :--- | :--- |
| `clifford_mhd_multivector.rs` | `j_vec.inner_product(&b_field)` | `electromagnetism::lorentz_force(j, b)` | Removes manual geometric product logic. |
| `clifford_mhd_multivector.rs` | (Manual safety check) | `EffectLog` (implicitly) | New function returns `PropagatingEffect` with logs. |
| `hilbert_multivector.rs` | `state.expectation_value(&op)` | `quantum::expectation_value(state, op)` | Standardizes observable measurement. |
| `hilbert_multivector.rs` | `state.bracket(&state)` | `quantum::born_probability(state)` | For normalization checks. |
| `differential_field.rs` | `manifold.laplacian(0)` | `topology::laplacian_scalar(manifold, 0)` | Wrapped in `PropagatingEffect`. |
| `differential_field.rs` | Manual Heat Eq: `u - dt * L * u` | (Optional) `thermodynamics::heat_diffusion` | Consider adding specific solver helper if recurring. |
| `grmhd_example/model.rs` | `compute_lorentz_force_internal` | `electromagnetism::lorentz_force(j, b)` | Eliminates custom helper function. |
| `grmhd_example/model.rs` | Manual G_uv calculation | `relativity::einstein_tensor(...)` | |
| `geometric_tilt_estimator.rs` | Manual angle calculations | `relativity::time_dilation_angle` or `waves::phase_velocity` | Depending on context (Rotational vs Relativistic). |

### 7.3. Action Steps

1.  **Refactor `grmhd_example` first**: It is the most complex monadic chain. Replacing its custom helpers with `deep_causality_physics` calls will prove the crate's utility.
2.  **Update `clifford_mhd`**: Show how `deep_causality_physics` maintains "Metric Agnosticism" automatically.
3.  **Standardize Quantum Examples**: Ensure `hilbert_multivector` uses the standard `quantum` module to avoid ad-hoc algebra usage.

## 8. Extension Consolidation Plan

**Goal:** Move `deep_causality_multivector/src/extensions/quantum` into `deep_causality_physics/src/quantum`.

1.  **Define Traits in Physics**: Copy `QuantumGates` and `QuantumOps` traits to `deep_causality_physics::quantum`.
2.  **Implement for HilbertState**: In `deep_causality_physics`, implement these traits for `deep_causality_multivector::HilbertState`.
    *   *Note:* `HilbertState` is foreign (from multivector) and Trait is local (to physics), so this is allowed.
3.  **Remove from Multivector**:
    *   Remove `deep_causality_multivector::extensions::quantum`.
    *   Update examples to import traits from `deep_causality_physics`.
4.  **Benefits**:
    *   Centralizes all Quantum Logic.
    *   Physics crate becomes the "Logic Layer" while Multivector remains the "Algebra Layer".


