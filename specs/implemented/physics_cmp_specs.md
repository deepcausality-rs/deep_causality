# Specification: DeepCausality Physics - Condensed Matter Module

## 1. Overview
This specification details the addition of a new `condensed` module to the `deep_causality_physics` crate. This module aims to provide rigorous, memory-safe implementations of high-performance physics kernels relevant to Condensed Matter Physics (CMP), specifically focusing on Quantum Geometry, Twistronics, and Topological Matter.

**Engineering Standard:** This specification adheres to high-assurance standards suitable for research facilities (e.g., CERN/Fermilab). This implies:
1.  **Dimensional Correctness:** Strict typing for physical quantities.
2.  **Numerical Stability:** Regularization for singularities (e.g., degenerate band gaps).
3.  **Traceability:** explicit mapping to theoretical models (Kang et al., Bistritzer-MacDonald).

The module will bridge the gap between theoretical Hamiltonians and causal reasoning by implementing:
1.  **Quantum Geometric Tensor (QGT) & Quasi-QGT**: Tools for analyzing the geometry of quantum states (Metric and Curvature).
2.  **Moiré & Strain Physics**: Bistritzer-MacDonald and Föppl–von Kármán equations.
3.  **Phase Field Models**: Ginzburg-Landau and Cahn-Hilliard equations for macroscopic order parameters.

## 2. Module Structure
The new module will reside in `src/condensed/` and follow the standard crate architecture:

```text
deep_causality_physics/
└── src/
    └── condensed/
        ├── mod.rs          # Public exports
        ├── qgt.rs          # Kernels for QGT, Quasi-QGT, Berry Curvature, Quantum Metric
        ├── moire.rs        # Kernels for Bistritzer-MacDonald and Strain
        ├── phase.rs        # Kernels for Ginzburg-Landau and Cahn-Hilliard
        ├── quantities.rs   # New physical quantities (Conductance, BerryPhase, etc.)
        └── wrappers.rs     # Causal wrappers (PropagatingEffect)
```

## 3. Data Types & Quantities
We will utilize existing types from `crate::units` and `crate::dynamics` where possible, and define new domain-specific quantities in `src/condensed/quantities.rs` using the **Newtype Pattern** to enforce strict type safety and prevent accidental unit mixing.

### 3.1 Existing Types Reused
*   `Energy` (from `crate::units::energy`)
*   `Time` (from `crate::units::time`)
*   `Ratio` (from `crate::units::ratio`)
*   `Length` (from `crate::dynamics::quantities`)
*   `Speed` (from `crate::dynamics::quantities`)
*   `Stiffness` (from `crate::materials::quantities`)

### 3.2 New Quantities (`src/condensed/quantities.rs`)

The following new types are defined to wrap specific scalar values or complex data structures (Tensors/MultiVectors) relevant to the condensed matter domain.

#### Scalar Wrappers
| Type Name | Inner Type | Physical Unit / Description |
| :--- | :--- | :--- |
| `QuantumMetric` | `f64` | Component of the metric tensor ($g_{ij}$). Dimensionless or $L^2$. |
| `BerryCurvature` | `f64` | Component of the curvature tensor ($\Omega_{ij}$). Area ($L^2$). |
| `BandDrudeWeight`| `f64` | Transport weight ($D$). |
| `OrbitalAngularMomentum` | `f64` | Intrinsic OAM ($L$). |
| `Conductance` | `f64` | Electrical conductance ($G$). Units: Siemens ($S$). |
| `Mobility` | `f64` | Charge carrier mobility ($\mu$). Units: $m^2/(V\cdot s)$. |
| `TwistAngle` | `f64` | Moiré twist angle ($\theta$). Radians. |
| `OrderParameter` | `Complex<f64>` | Superconducting/Phase order parameter ($\psi$). |

#### Data Structure Wrappers
| Type Name | Inner Type | Description |
| :--- | :--- | :--- |
| `QuantumEigenvector` | `CausalTensor<Complex<f64>>` | State vector $|u_n\rangle$. Rank 2 (States $\times$ Basis). |
| `QuantumVelocity` | `CausalTensor<Complex<f64>>` | Velocity vector $\partial_i H |u_n\rangle$. |
| `Momentum` | `CausalMultiVector<f64>` | Crystal momentum vector $\mathbf{k}$. |
| `Displacement` | `CausalTensor<f64>` | Displacement field $\mathbf{u}(\mathbf{r})$. |
| `Concentration` | `CausalTensor<f64>` | Concentration field $c(\mathbf{r})$. |
| `ChemicalPotentialGradient` | `CausalTensor<f64>` | Gradient field $\nabla \mu$. |
| `VectorPotential` | `CausalMultiVector<f64>` | Electromagnetic Vector Potential $\mathbf{A}$. |

### 3.3 Implementation of New Quantities

```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, PhysicsErrorEnum};
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;
use deep_causality_multivector::{CausalMultiVector, Metric};

// ============================================================================ 
// Scalar Wrappers
// ============================================================================ 

/// Quantum Metric component ($g_{ij}$). 
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct QuantumMetric(f64);

impl QuantumMetric {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        // Metric components can be negative (off-diagonal), so no invariant check here.
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<QuantumMetric> for f64 {
    fn from(val: QuantumMetric) -> Self {
        val.0
    }
}

/// Berry Curvature component ($\Omega_{ij}$). 
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct BerryCurvature(f64);

impl BerryCurvature {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<BerryCurvature> for f64 {
    fn from(val: BerryCurvature) -> Self {
        val.0
    }
}

/// Band Drude Weight ($D$). 
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct BandDrudeWeight(f64);

impl BandDrudeWeight {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Orbital Angular Momentum ($L$). 
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct OrbitalAngularMomentum(f64);

impl OrbitalAngularMomentum {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Electrical Conductance ($G$). 
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Conductance(f64);

impl Conductance {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
             return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Conductance".into()),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Charge Carrier Mobility ($\mu$). 
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Mobility(f64);

impl Mobility {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        // Mobility is typically magnitude, thus non-negative.
        if val < 0.0 {
             return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Mobility".into()),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Moiré Twist Angle ($\theta$). 
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct TwistAngle(f64);

impl TwistAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
    pub fn as_degrees(&self) -> f64 {
        self.0.to_degrees()
    }
    pub fn from_degrees(deg: f64) -> Self {
        Self(deg.to_radians())
    }
}

/// Superconducting Order Parameter ($\psi$). 
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OrderParameter(Complex<f64>);

impl OrderParameter {
    pub fn new(val: Complex<f64>) -> Self {
        Self(val)
    }
    pub fn value(&self) -> Complex<f64> {
        self.0
    }
    pub fn magnitude_squared(&self) -> f64 {
        self.0.norm_sqr()
    }
}

// ============================================================================ 
// Data Structure Wrappers
// ============================================================================ 

/// Wrapper for a Quantum Eigenvector $|u_n\rangle$. 
/// Expected to be Rank 2 Tensor [basis_size, num_states].
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumEigenvector(CausalTensor<Complex<f64>>);

impl QuantumEigenvector {
    pub fn new(tensor: CausalTensor<Complex<f64>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<f64>> {
        &self.0
    }
}

/// Wrapper for a Quantum Velocity vector $\partial_i H |u_n\rangle$. 
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumVelocity(CausalTensor<Complex<f64>>);

impl QuantumVelocity {
    pub fn new(tensor: CausalTensor<Complex<f64>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<f64>> {
        &self.0
    }
}

/// Wrapper for Momentum vector $\mathbf{k}$. 
#[derive(Debug, Clone, PartialEq)]
pub struct Momentum(CausalMultiVector<f64>);

impl Default for Momentum {
    fn default() -> Self {
        Self(CausalMultiVector::new(vec![0.0], Metric::Euclidean(0)).unwrap())
    }
}

impl Momentum {
    pub fn new(mv: CausalMultiVector<f64>) -> Self {
        Self(mv)
    }
    pub fn inner(&self) -> &CausalMultiVector<f64> {
        &self.0
    }
}

/// Wrapper for Displacement field $\mathbf{u}(\mathbf{r})$. 
#[derive(Debug, Clone, PartialEq)]
pub struct Displacement(CausalTensor<f64>);

impl Displacement {
    pub fn new(tensor: CausalTensor<f64>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Wrapper for Concentration field $c(\mathbf{r})$. 
#[derive(Debug, Clone, PartialEq)]
pub struct Concentration(CausalTensor<f64>);

impl Concentration {
    pub fn new(tensor: CausalTensor<f64>) -> Result<Self, PhysicsError> {
        // Concentration cannot be negative
        for &val in tensor.as_slice() {
            if val < 0.0 {
                return Err(PhysicsError::new(
                    PhysicsErrorEnum::PhysicalInvariantBroken("Negative Concentration detected".into())
                ));
            }
        }
        Ok(Self(tensor))
    }
    /// Creates a new Concentration without validation.
    /// Use only if the tensor is guaranteed to be non-negative.
    pub fn new_unchecked(tensor: CausalTensor<f64>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Wrapper for Chemical Potential Gradient $\nabla \mu$. 
#[derive(Debug, Clone, PartialEq)]
pub struct ChemicalPotentialGradient(CausalTensor<f64>);

impl ChemicalPotentialGradient {
    pub fn new(tensor: CausalTensor<f64>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Wrapper for Electromagnetic Vector Potential $\mathbf{A}$. 
#[derive(Debug, Clone, PartialEq)]
pub struct VectorPotential(CausalMultiVector<f64>);

impl Default for VectorPotential {
    fn default() -> Self {
        Self(CausalMultiVector::new(vec![0.0], Metric::Euclidean(0)).unwrap())
    }
}

impl VectorPotential {
    pub fn new(mv: CausalMultiVector<f64>) -> Self {
        Self(mv)
    }
    pub fn inner(&self) -> &CausalMultiVector<f64> {
        &self.0
    }
}
```

## 4. Kernel Specifications

### 4.1 Quantum Geometry (`src/condensed/qgt.rs`)
Based on *Kang et al. (2412.17809)*.

#### 4.1.1 `quantum_geometric_tensor_kernel`
Calculates the QGT component $Q_{ij}^n(\mathbf{k})$ for band $n$.
Standard formula with regularization for numerical stability (avoiding division by zero in degeneracies).

*   **Inputs**:
    *   `eigenvalues`: `&CausalTensor<f64>` ($E_n$)
    *   `eigenvectors`: `&QuantumEigenvector` ($|u_n\rangle$, matrix columns)
    *   `velocity_i`: `&QuantumVelocity` ($\partial_i H$)
    *   `velocity_j`: `&QuantumVelocity` ($\partial_j H$)
    *   `band_n`: `usize` (Target band index)
    *   `regularization`: `f64` (Small $\epsilon$ for $(E_n - E_m)^2 + \epsilon$)
*   **Returns**: `Result<Complex<f64>, PhysicsError>` (The scalar component $Q_{ij}$)

#### 4.1.2 `quasi_qgt_kernel`
Calculates the Quasi-QGT $q_{ij}^n(\mathbf{k})$.

*   **Inputs**: Same as QGT (using new types) + `regularization`.
*   **Returns**: `Result<Complex<f64>, PhysicsError>`

#### 4.1.3 `effective_band_drude_weight_kernel`
Approximation of BDW using band curvature.

*   **Inputs**:
    *   `energy_n`: `Energy`
    *   `energy_0`: `Energy`
    *   `curvature_ii`: `f64`
    *   `lattice_const`: `Length`
*   **Returns**: `Result<BandDrudeWeight, PhysicsError>`

### 4.2 Moiré Physics (`src/condensed/moire.rs`)

#### 4.2.1 `bistritzer_macdonald_kernel`
Constructs the continuum Hamiltonian for TBG. This requires defining a momentum cutoff.

*   **Inputs**:
    *   `twist_angle`: `TwistAngle`
    *   `interlayer_coupling`: `Energy`
    *   `fermi_velocity`: `Speed`
    *   `k_point`: `Momentum`
    *   `shell_cutoff`: `usize` (Number of Moiré shells to include in basis)
*   **Returns**: `Result<CausalTensor<Complex<f64>>, PhysicsError>`

#### 4.2.2 `foppl_von_karman_strain_kernel`
Calculates stress tensor given displacement fields.

*   **Inputs**:
    *   `displacement_u`: `Displacement` (In-plane)
    *   `displacement_w`: `Displacement` (Out-of-plane)
    *   `youngs_modulus`: `Stiffness`
    *   `poisson_ratio`: `Ratio`
*   **Returns**: `Result<CausalTensor<f64>, PhysicsError>` (Stress Tensor)

### 4.3 Phase Fields (`src/condensed/phase.rs`)

#### 4.3.1 `ginzburg_landau_free_energy_kernel`
Calculates free energy density with optional magnetic coupling.

*   **Inputs**:
    *   `psi`: `OrderParameter`
    *   `alpha`: `f64`
    *   `beta`: `f64`
    *   `gradient_psi`: `CausalMultiVector<Complex<f64>>`
    *   `vector_potential`: `Option<&VectorPotential>` (Coupling term $\mathbf{A}$)
*   **Returns**: `Result<Energy, PhysicsError>`

#### 4.3.2 `cahn_hilliard_flux_kernel`
Calculates the flux $J$.

*   **Inputs**:
    *   `concentration`: `Concentration`
    *   `mobility`: `Mobility`
    *   `chem_potential_grad`: `ChemicalPotentialGradient`
*   **Returns**: `Result<CausalTensor<f64>, PhysicsError>` (Flux Tensor)

## 5. Causal Wrappers (`src/condensed/wrappers.rs`)
Implement `PropagatingEffect<T>` return types for all kernels above using the new quantities.

## 6. Implementation Strategy (Detailed Plan)

This section maps the theoretical concepts from the `deep_causality_haft` traits and the QGT Physics notes to the implementation strategy.

### 6.1 Architecture: The Brillouin Monad
We will treat the Brillouin Zone (Momentum Space) as a topological manifold where we can perform "Comonadic Extension" to extract gradients.

*   **Data Structure**: Use `deep_causality_topology::Manifold<Complex<f64>>` to represent the discretized Bloch states over the Brillouin Zone.
*   **Pattern**: Implement a **Bounded Comonad** pattern (conceptually, via `deep_causality_haft::BoundedComonad` if applicable, or a specific `extract`/`extend` implementation) to compute the discrete derivative of the wavefunction.

### 6.2 Topological Gradient Calculation
To calculate the QGT, we need $\partial_{\mathbf{k}} |u_{n\mathbf{k}}\rangle$.

1.  **Input**: A `Manifold` where each vertex stores a `QuantumEigenvector` (wrapping `CausalTensor` or `CausalMultiVector`).
2.  **Coboundary Operator**: Use the topological `coboundary` (discrete exterior derivative) to compute the change in state between neighboring $k$-points.
    *   $|d u\rangle \approx |u(k + \delta k)\rangle - |u(k)\rangle$.
3.  **Gauge Invariance**: Ensure the calculation respects the U(1) gauge freedom (phase factors) inherent in quantum states.

### 6.3 Geometric Algebra Integration (The QGT Construction)
We will leverage `deep_causality_multivector` to compute the QGT components naturally.

*   **The Formula**: $Q_{ij} = \langle \partial_i u | (1 - |u\rangle\langle u|) | \partial_j u \rangle$.
*   **GA Equivalent**: The **Geometric Product** of the state gradients naturally separates the Metric and Curvature.
    *   Let $\psi$ be the spinor representation of the state.
    *   Compute gradients $\nabla \psi$.
    *   Compute the geometric product: $P = (\nabla \psi)^\dagger (\nabla \psi)$.
    *   **Quantum Metric ($g_{ij}$)**: The **Scalar Part** (Real) of $P$.
    *   **Berry Curvature ($\Omega_{ij}$)**: The **Bivector Part** (Imaginary) of $P$.

### 6.4 Quasi-QGT & Observables
The implementation will explicitly map the abstract tensor components to physical observables as defined in Kang et al.

*   **Kernel**: `quasi_qgt_kernel`.
*   **Mapping**:
    *   Real Part $\to$ `BandDrudeWeight`.
    *   Imaginary Part $\to$ `OrbitalAngularMomentum`.
*   **Usage**: This allows direct comparison with CD-ARPES experimental data (which measures OAM).

### 6.5 Adjunction between Topology and Algebra
We will define an `Adjunction` (using `deep_causality_haft::Adjunction`) to formalize the relationship between the **Global Topology** (Manifold/Chern Number) and the **Local Geometry** (QGT/Metric).

*   **Left Functor ($L$)**: The Geometric Construction (building the local QGT metric from the manifold).
*   **Right Functor ($R$)**: The Topological Invariant (extracting the global Chern number from the curvature field).
*   This structure allows `DeepCausality` to reason about "Topological Protection" causally: if the Chern number (Right) is non-zero, it *causes* the robust edge states (Left).

## 7. Test Plan (Aiming for 100% Coverage)
Updated to instantiate new types in tests.

### 7.1 Test files 

```text
deep_causality_physics/
└── src/
    └── condensed/
        ├── mod.rs          
        ├── qgt_tests.rs    
        ├── moire_tests.rs        
        ├── phase_tests.rs        
        ├── quantities_tests.rs   
        └── wrappers_tests.rs    
```


## 8. Dependencies
*   `deep_causality_tensor`
*   `deep_causality_multivector`
*   `deep_causality_core`
*   `deep_causality_num`
*   `deep_causality_topology` (For Manifold/Coboundary)
*   `deep_causality_haft` (For Comonad/Adjunction traits)
