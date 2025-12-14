# Specification: DeepCausality Physics - Photonics Module

## 1. Overview
This specification details the addition of a new `photonics` module to the `deep_causality_physics` crate. This module provides a rigorous, type-safe implementation of classical and quantum optics, focusing on **Ray Optics**, **Polarization Calculus**, **Gaussian Beam Optics**, and **Diffraction**.

**Engineering Standard:** Adheres to high-assurance standards (CERN/Fermilab grade).
*   **Type Safety:** Newtype pattern for all optical quantities to prevent dimensional analysis errors (e.g., mixing Focal Length with Path Length).
*   **Numerical Stability:** Robust handling of singularities (e.g., critical angles in Snell's law, resonator stability edges).
*   **Algebraic Structure:** Utilization of Geometric Algebra (via `CausalMultiVector`) and Tensor Algebra (`CausalTensor`) for polarization and ray transfer matrices.

## 2. Module Structure
The module will reside in `src/photonics/` following the crate's architecture:

```text
deep_causality_physics/
└── src/
    └── photonics/
        ├── mod.rs          # Public exports
        ├── ray.rs          # Matrix Optics (ABCD), Snell's Law
        ├── polarization.rs # Jones & Mueller Calculus
        ├── beam.rs         # Gaussian Beam propagation (q-parameter)
        ├── diffraction.rs  # Fresnel/Fraunhofer & Grating kernels
        ├── quantities.rs   # Domain-specific newtypes (FocalLength, etc.)
        └── wrappers.rs     # Causal wrappers (PropagatingEffect)
```

## 3. Data Types & Quantities

We reuse `Length`, `Speed` (light speed), `Frequency`, and `IndexOfRefraction` from core modules. We define specific optical quantities to enforce semantic correctness.

### 3.1 New Quantities (`src/photonics/quantities.rs`)

```rust
use crate::{PhysicsError, PhysicsErrorEnum};
use deep_causality_tensor::CausalTensor;
use deep_causality_num::Complex;

/// Focal Length ($f$).
/// Unit: Meters. Constraint: None (can be negative for diverging lens).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct FocalLength(f64);

impl FocalLength {
    pub fn new(val: f64) -> Result<Self, PhysicsError> { Ok(Self(val)) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Optical Power ($D = 1/f$).
/// Unit: Diopters ($m^{-1}$). Constraint: None.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct OpticalPower(f64);

impl OpticalPower {
    pub fn new(val: f64) -> Result<Self, PhysicsError> { Ok(Self(val)) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Wavelength ($\lambda$).
/// Unit: Meters. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Wavelength(f64);

impl Wavelength {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Wavelength must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Numerical Aperture ($NA = n \sin \theta$).
/// Unit: Dimensionless. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct NumericalAperture(f64);

impl NumericalAperture {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Numerical Aperture must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Beam Waist ($w_0$). Minimum radius of Gaussian beam.
/// Unit: Meters. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct BeamWaist(f64);

impl BeamWaist {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Beam Waist must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self { Self(val) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Ray Height ($y$). Distance from optical axis.
/// Unit: Meters.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RayHeight(f64);

impl RayHeight {
    pub fn new(val: f64) -> Result<Self, PhysicsError> { Ok(Self(val)) }
    pub fn value(&self) -> f64 { self.0 }
}

/// Ray Angle ($\theta$). Angle relative to optical axis.
/// Unit: Radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RayAngle(f64);

impl RayAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> { Ok(Self(val)) }
    pub fn value(&self) -> f64 { self.0 }
}

/// ABCD Matrix. $2 \times 2$ Ray Transfer Matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct AbcdMatrix(CausalTensor<f64>);

impl AbcdMatrix {
    pub fn new(tensor: CausalTensor<f64>) -> Self { Self(tensor) }
    pub fn inner(&self) -> &CausalTensor<f64> { &self.0 }
}

/// Jones Vector. Polarized Electric Field. Rank 1, Dim 2 Complex Tensor.
#[derive(Debug, Clone, PartialEq)]
pub struct JonesVector(CausalTensor<Complex<f64>>);

impl JonesVector {
    pub fn new(tensor: CausalTensor<Complex<f64>>) -> Self { Self(tensor) }
    pub fn inner(&self) -> &CausalTensor<Complex<f64>> { &self.0 }
}

/// Stokes Vector. Intensity vector $(S_0, S_1, S_2, S_3)$. Rank 1, Dim 4 Tensor.
/// Constraint: $S_0^2 \ge S_1^2 + S_2^2 + S_3^2$.
#[derive(Debug, Clone, PartialEq)]
pub struct StokesVector(CausalTensor<f64>);

impl StokesVector {
    pub fn new(tensor: CausalTensor<f64>) -> Self { Self(tensor) }
    pub fn inner(&self) -> &CausalTensor<f64> { &self.0 }
}

/// Complex Beam Parameter ($q(z) = z + i z_R$).
/// Constraint: $\text{Im}(q) > 0$.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ComplexBeamParameter(Complex<f64>);

impl ComplexBeamParameter {
    pub fn new(val: Complex<f64>) -> Result<Self, PhysicsError> {
        if val.im <= 0.0 {
             return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Imaginary part of q (Rayleigh range) must be positive".into()
            )));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: Complex<f64>) -> Self { Self(val) }
    pub fn value(&self) -> Complex<f64> { self.0 }
}
```

## 4. Kernel Specifications

### 4.1 Ray Optics (`src/photonics/ray.rs`)

#### 4.1.1 `ray_transfer_kernel`
Applies an ABCD matrix to a ray vector.
$$ \begin{pmatrix} y_{out} \ \theta_{out} \end{pmatrix} = \begin{pmatrix} A & B \ C & D \end{pmatrix} \begin{pmatrix} y_{in} \ \theta_{in} \end{pmatrix} $$

```rust
pub fn ray_transfer_kernel(
    matrix: &AbcdMatrix, 
    height: RayHeight, 
    angle: RayAngle
) -> Result<(RayHeight, RayAngle), PhysicsError>;
```

#### 4.1.2 `snells_law_kernel`
Calculates refracted angle or Critical Angle error.
$$ n_1 \sin \theta_1 = n_2 \sin \theta_2 $$

```rust
pub fn snells_law_kernel(
    n1: IndexOfRefraction, 
    n2: IndexOfRefraction, 
    theta1: RayAngle
) -> Result<RayAngle, PhysicsError>;
```

#### 4.1.3 `lens_maker_kernel`
Calculates optical power/focal length from curvature.
$$ P = (n - 1) \left( \frac{1}{R_1} - \frac{1}{R_2} \right) $$

```rust
pub fn lens_maker_kernel(
    n: IndexOfRefraction, 
    r1: Length, // Front radius
    r2: Length  // Back radius
) -> Result<OpticalPower, PhysicsError>;
```

### 4.2 Polarization (`src/photonics/polarization.rs`)

#### 4.2.1 `jones_rotation_kernel`
Rotates an optical element's axis by angle $\phi$.
$$ J'(\phi) = R(-\phi) \cdot J \cdot R(\phi) $$
where $R(\phi)$ is the 2D rotation matrix. Used to rotate the *operator* (Jones Matrix).

```rust
pub fn jones_rotation_kernel(
    jones_matrix: &CausalTensor<Complex<f64>>, 
    angle: RayAngle
) -> Result<CausalTensor<Complex<f64>>, PhysicsError>;
```

#### 4.2.2 `stokes_from_jones_kernel`
Converts pure state Jones vector to Stokes vector.
$$ S_0 = |E_x|^2 + |E_y|^2, \quad S_1 = |E_x|^2 - |E_y|^2, \quad S_2 = 2\text{Re}(E_x E_y^*), \quad S_3 = -2\text{Im}(E_x E_y^*) $$

```rust
pub fn stokes_from_jones_kernel(
    jones: &JonesVector
) -> Result<StokesVector, PhysicsError>;
```

#### 4.2.3 `degree_of_polarization_kernel`
$$ DOP = \frac{\sqrt{S_1^2 + S_2^2 + S_3^2}}{S_0} $$

```rust
pub fn degree_of_polarization_kernel(
    stokes: &StokesVector
) -> Result<Ratio, PhysicsError>;
```

### 4.3 Gaussian Beam Optics (`src/photonics/beam.rs`)

#### 4.3.1 `gaussian_q_propagation_kernel`
Propagates the complex beam parameter $q$ through an ABCD system.
$$ q_{out} = \frac{A q_{in} + B}{C q_{in} + D} $$

```rust
pub fn gaussian_q_propagation_kernel(
    q_in: ComplexBeamParameter, 
    matrix: &AbcdMatrix
) -> Result<ComplexBeamParameter, PhysicsError>;
```

#### 4.3.2 `beam_spot_size_kernel`
Extracts spot size $w(z)$ from $q$.
$$ \frac{1}{q} = \frac{1}{R(z)} - i \frac{\lambda}{\pi w(z)^2} $$
Therefore, $w(z) = \sqrt{\frac{-\lambda}{\pi \text{Im}(1/q)}}$.

```rust
pub fn beam_spot_size_kernel(
    q: ComplexBeamParameter, 
    wavelength: Wavelength
) -> Result<Length, PhysicsError>;
```

### 4.4 Diffraction (`src/photonics/diffraction.rs`)

#### 4.4.1 `single_slit_irradiance_kernel`
Fraunhofer diffraction pattern.
$$ I(\theta) = I_0 \left( \frac{\sin \beta}{\beta} \right)^2, \quad \beta = \frac{\pi a \sin \theta}{\lambda} $$

```rust
pub fn single_slit_irradiance_kernel(
    i0: f64, 
    slit_width: Length, 
    theta: RayAngle, 
    wavelength: Wavelength
) -> Result<f64, PhysicsError>;
```

#### 4.4.2 `grating_equation_kernel`
Finds angle of $m$-th order maximum.
$$ d (\sin \theta_m - \sin \theta_i) = m \lambda $$

```rust
pub fn grating_equation_kernel(
    pitch: Length, 
    order: i32, 
    incidence: RayAngle, 
    wavelength: Wavelength
) -> Result<RayAngle, PhysicsError>;
```

## 5. Causal Wrappers (`src/photonics/wrappers.rs`)

```rust
use crate::photonics::{ray, polarization, beam, diffraction, quantities::*};
use deep_causality_core::{CausalityError, PropagatingEffect};

pub fn ray_transfer(m: &AbcdMatrix, h: RayHeight, a: RayAngle) -> PropagatingEffect<(RayHeight, RayAngle)> {
    match ray::ray_transfer_kernel(m, h, a) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn snells_law(n1: IndexOfRefraction, n2: IndexOfRefraction, theta1: RayAngle) -> PropagatingEffect<RayAngle> {
    match ray::snells_law_kernel(n1, n2, theta1) {
        Ok(theta2) => PropagatingEffect::pure(theta2),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ... Additional wrappers for beam, polarization, diffraction ...
```

## 6. Higher-Kinded Types (HKT) Integration

We define a **Polarization Map** concept using HKTs to abstract operations that apply to both Jones (Amplitude/Coherent) and Mueller (Intensity/Incoherent) domains.

*   **Concept**: `PolarizationOp<V>` where `V` is the vector type (`JonesVector` or `StokesVector`).
*   This allows defining generic optical elements (e.g., "Linear Polarizer at axis $\theta$") that can generate either the $2\times2$ Jones matrix or $4\times4$ Mueller matrix depending on the simulation context.

## 7. Testing Strategy

*   **Unit Tests**: Verify Snell's law critical angle failure, ABCD matrix multiplication (identity, cascade), and Gaussian beam stability.
*   **Invariant Tests**:
    *   $S_0^2 \ge S_1^2 + S_2^2 + S_3^2$ for Stokes vectors.
    *   $\text{Im}(q) > 0$ for Gaussian beams.
*   **Integration Tests**: Simulate a simple laser cavity (Round trip $q_{out} = q_{in}$) to calculate mode stability.

## 8. Dependencies
*   `deep_causality_tensor` (for Matrix operations).
*   `deep_causality_num` (for Complex numbers).
*   `deep_causality_multivector` (optional, for field representations if needed, though Tensor is sufficient for standard Matrix Optics).

```