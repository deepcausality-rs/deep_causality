# Specification: DeepCausality Physics - MHD/GRMHD Module

## 1. Overview
This specification details the addition of a new `mhd` module to the `deep_causality_physics` crate. This module provides a rigorous, type-safe implementation of Magnetohydrodynamics (MHD) and General Relativistic Magnetohydrodynamics (GRMHD), enabling simulations of astrophysical plasmas (e.g., accretion disks, jets) and fusion reactors.

**Engineering Standard:** Adheres to high-assurance standards (CERN/Fermilab grade).
*   **Metric Agnosticism:** Core kernels operate on `CausalMultiVector` and `CausalTensor`, automatically adapting to Euclidean (Classical) or Minkowski/Curved (Relativistic) metrics.
*   **Type Safety:** Newtype pattern for all physical quantities.
*   **Coupling:** Explicit coupling between Fluid Dynamics (Navier-Stokes/Euler) and Electromagnetism (Maxwell) via Lorentz Force and Induction equations.

## 2. Module Structure
The module will reside in `src/mhd/` following the crate's architecture:

```text
deep_causality_physics/
└── src/
    └── mhd/
        ├── mod.rs          # Public exports
        ├── ideal.rs        # Ideal MHD (Infinite Conductivity)
        ├── resistive.rs    # Resistive MHD (Finite Conductivity)
        ├── grmhd.rs        # General Relativistic MHD (Metric Coupling)
        ├── plasma.rs       # Plasma Parameters (Debye, Larmor, etc.)
        ├── quantities.rs   # Domain-specific newtypes
        └── wrappers.rs     # Causal wrappers (PropagatingEffect)
```

## 3. Data Types & Quantities

Reusing `Density`, `Pressure`, `Velocity` (Speed), `MagneticField` (PhysicalField), `CurrentDensity` from core physics modules.

### 3.1 New Quantities (`src/mhd/quantities.rs`)

The following types enforce physical invariants (e.g., non-negative pressure) and prevent dimensional errors.

```rust
use crate::{PhysicsError, PhysicsErrorEnum};

/// Alfven Speed ($v_A$). Characteristic speed of magnetic waves in plasma.
/// Unit: m/s. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct AlfvenSpeed(f64);

impl AlfvenSpeed {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Alfven Speed cannot be negative".into()
            )));
        }
        Ok(Self(val))
    }
    /// Creates a new `AlfvenSpeed` without validation.
    /// Use only if the value is guaranteed to be non-negative.
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Plasma Beta ($eta$). Ratio of thermal to magnetic pressure.
/// Unit: Dimensionless. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct PlasmaBeta(f64);

impl PlasmaBeta {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Plasma Beta cannot be negative".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Magnetic Pressure ($P_B$). Energy density of the magnetic field.
/// Unit: Pascals (Pa). Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct MagneticPressure(f64);

impl MagneticPressure {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Magnetic Pressure cannot be negative".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Larmor Radius ($r_L$). Gyroradius of a charged particle.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct LarmorRadius(f64);

impl LarmorRadius {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Larmor Radius must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Debye Length ($\lambda_D$). Screening length in plasma.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct DebyeLength(f64);

impl DebyeLength {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Debye Length must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Plasma Frequency ($\omega_{pe}$). Natural oscillation frequency.
/// Unit: Rad/s. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct PlasmaFrequency(f64);

impl PlasmaFrequency {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Plasma Frequency must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Electrical Conductivity ($\sigma$).
/// Unit: Siemens/m (S/m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Conductivity(f64);

impl Conductivity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Conductivity must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Magnetic Diffusivity ($\eta$).
/// Unit: $m^2/s$. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Diffusivity(f64);

impl Diffusivity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Diffusivity cannot be negative".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}
```

## 4. Kernel Specifications

### 4.1 Ideal MHD (`src/mhd/ideal.rs`)

#### 4.1.1 `alfven_speed_kernel`
Calculates the characteristic speed of Alfven waves.
$$ v_A = \frac{B}{\sqrt{\mu_0 \rho}} $$

```rust
pub fn alfven_speed_kernel(
    b_field: &PhysicalField, 
    density: &Density, 
    permeability: f64
) -> Result<AlfvenSpeed, PhysicsError>;
```

#### 4.1.2 `magnetic_pressure_kernel`
Calculates magnetic pressure.
$$ P_B = \frac{B^2}{2\mu_0} $$

```rust
pub fn magnetic_pressure_kernel(
    b_field: &PhysicalField, 
    permeability: f64
) -> Result<MagneticPressure, PhysicsError>;
```

#### 4.1.3 `ideal_induction_kernel`
Calculates the time evolution of the magnetic field (Frozen-in flux).
$$ \frac{\partial \mathbf{B}}{\partial t} = \nabla \times (\mathbf{v} \times \mathbf{B}) $$

**Geometric Algebra Implementation**:
In the language of differential forms/GA on a Manifold:
$$ \partial_t B = -d(i_v B) $$
where $B$ is a 2-form (flux), $v$ is a vector field, $i_v$ is interior product (contraction), and $d$ is exterior derivative. This ensures $\nabla \cdot B = 0$ is preserved.

```rust
pub fn ideal_induction_kernel(
    v_manifold: &Manifold<f64>, 
    b_manifold: &Manifold<f64>
) -> Result<CausalTensor<f64>, PhysicsError>;
```

### 4.2 Resistive MHD (`src/mhd/resistive.rs`)

#### 4.2.1 `resistive_diffusion_kernel`
Calculates the diffusion term of the induction equation.
$$ \frac{\partial \mathbf{B}}{\partial t}_{diff} = \eta \nabla^2 \mathbf{B} $$
On a manifold, this is $-\eta \Delta B$ where $\Delta = d\delta + \delta d$ is the Hodge Laplacian.

```rust
pub fn resistive_diffusion_kernel(
    b_manifold: &Manifold<f64>, 
    diffusivity: Diffusivity
) -> Result<CausalTensor<f64>, PhysicsError>;
```

#### 4.2.2 `magnetic_reconnection_rate_kernel`
Estimates reconnection rate (Sweet-Parker model simplified).
$$ v_{in} = \frac{v_A}{\sqrt{S}} $$
where $S$ is the Lundquist number.

```rust
pub fn magnetic_reconnection_rate_kernel(
    alfven_speed: AlfvenSpeed, 
    lundquist: f64
) -> Result<Speed, PhysicsError>;
```

### 4.3 GRMHD (`src/mhd/grmhd.rs`)

#### 4.3.1 `relativistic_current_kernel`
Calculates current density $J^\mu$ compatible with curved spacetime.
$$ J^\mu = \nabla_\nu F^{\mu\nu} $$
(Divergence of Electromagnetic Tensor).

```rust
pub fn relativistic_current_kernel(
    em_tensor: &CausalTensor<f64>, 
    metric: &CausalTensor<f64>
) -> Result<CausalTensor<f64>, PhysicsError>;
```

#### 4.3.2 `energy_momentum_tensor_em_kernel`
Calculates the electromagnetic stress-energy tensor $T^{\mu\nu}_{EM}$.
$$ T^{\mu\nu} = F^{\mu\alpha}F^\nu_\alpha - \frac{1}{4} g^{\mu\nu} F_{\alpha\beta}F^{\alpha\beta} $$

```rust
pub fn energy_momentum_tensor_em_kernel(
    em_tensor: &CausalTensor<f64>, 
    metric: &CausalTensor<f64>
) -> Result<CausalTensor<f64>, PhysicsError>;
```

### 4.4 Plasma Parameters (`src/mhd/plasma.rs`)

#### 4.4.1 `debye_length_kernel`
$$ \lambda_D = \sqrt{\frac{\epsilon_0 k_B T_e}{n_e e^2}} $$

```rust
pub fn debye_length_kernel(
    temp: Temperature, 
    density_n: f64, 
    epsilon_0: f64, 
    elementary_charge: f64
) -> Result<DebyeLength, PhysicsError>;
```

#### 4.4.2 `larmor_radius_kernel`
$$ r_L = \frac{m v_\perp}{|q| B} $$

```rust
pub fn larmor_radius_kernel(
    mass: Mass, 
    velocity_perp: Speed, 
    charge: f64, 
    b_field: &PhysicalField
) -> Result<LarmorRadius, PhysicsError>;
```

## 5. Causal Wrappers (`src/mhd/wrappers.rs`)

```rust
use crate::mhd::{ideal, resistive, grmhd, plasma, quantities::*};
use deep_causality_core::{CausalityError, PropagatingEffect};

pub fn alfven_speed(b: &PhysicalField, rho: &Density, mu0: f64) -> PropagatingEffect<AlfvenSpeed> {
    match ideal::alfven_speed_kernel(b, rho, mu0) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn magnetic_pressure(b: &PhysicalField, mu0: f64) -> PropagatingEffect<MagneticPressure> {
    match ideal::magnetic_pressure_kernel(b, mu0) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn ideal_induction(v: &Manifold<f64>, b: &Manifold<f64>) -> PropagatingEffect<CausalTensor<f64>> {
    match ideal::ideal_induction_kernel(v, b) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ... Additional wrappers for resistive, grmhd, plasma kernels ...
```

## 6. Higher-Kinded Types (HKT) Integration

We define an **Equation of State (EOS)** abstraction using HKTs to allow the MHD solver to switch between thermodynamic closures (Isothermal, Adiabatic, Polytropic) without changing the conservation laws.

*   **Concept**: `EosWitness` implementing `HKT` for `Pressure = F(Density, InternalEnergy)`.
*   This integration ensures the solver is generic over the thermodynamic behavior of the plasma.

## 7. Testing Strategy

*   **Unit Tests**: Verify Alfven speed limits, Magnetic pressure positivity.
*   **Integration Tests**:
    *   **Ideal**: Verify B-field conservation in zero velocity field.
    *   **GRMHD**: Verify $T^{\mu\nu}$ symmetry and trace conditions (Trace $T=0$ for EM field).
*   **Metric Checks**: Ensure GRMHD kernels fail if metric signature is inconsistent.

## 8. Dependencies
*   `deep_causality_tensor` (Tensor operations).
*   `deep_causality_multivector` (Geometric Algebra).
*   `deep_causality_topology` (Manifold gradients).