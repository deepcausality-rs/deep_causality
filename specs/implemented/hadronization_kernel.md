# Hadronization Kernel Specification

**Status:** Draft
**Context:** High Energy Physics, Multi-Physics Pipelines (scalar to vector field transition).

## 1. Overview
The `hadronization_kernel` simulates the process of hadronization, where high-energy scalar fields (e.g., quark-gluon plasma or Higgs decay products) transform into bounded vector particles (hadrons/jets). In the context of valid causal multi-physics pipelines, this serves as a critical bridge between Scalar Field Theory (Simplicial Manifold) and Particle Dynamics (Graph/Vector).

## 2. Mathematical Definition
The process maps a scalar energy density field $\phi(x)$ or $\rho(x)$ to a set of momentum vectors $\vec{p}_i$ or a vector field $V^\mu(x)$.

**Simplified Model (Lund String Model inspired):**
Isolate regions of high energy density $\rho > \rho_{crit}$.
For each region, generate particle momenta $\vec{p}$ such that:
$$ \sum |\vec{p}_i| \approx \int \rho(x) dV $$
(Energy Conservation)

## 3. Implementation Details

## 3. Implementation Details

### Required Types
*   **`EnergyDensity`**: New struct to be added in `deep_causality_physics/src/nuclear/quantities.rs`.
    *   Wraps `f64`.
    *   Validation: Non-negative.

### Function Signature
```rust
use crate::EnergyDensity;
use crate::PhysicalVector;

pub fn hadronization_kernel(
    energy_density: &[EnergyDensity],
    threshold: EnergyDensity,
    dim: usize
) -> Result<Vec<PhysicalVector>, PhysicsError>
```

### Logic
1.  **Thresholding:** Filter input indices where `energy_density[i].value() > threshold.value()`.
2.  **Vectorization:**
    *   Magnitude proportional to `energy_density[i].value()`.
    *   ...

## 4. Verification
*   **Conservation:** Sum of vector magnitudes should be proportional to scalar energy.

## 5. Integration

### Location
*   **Append:** `deep_causality_physics/src/nuclear/physics.rs`

### Wrapper Implementation
```rust
use deep_causality_core::{CausalityError, PropagatingEffect};
use crate::nuclear::physics;
use crate::EnergyDensity;
use crate::PhysicalVector;

pub fn hadronization(
    energy_density: &[EnergyDensity],
    threshold: EnergyDensity,
    dim: usize
) -> PropagatingEffect<Vec<PhysicalVector>>
{
    match physics::hadronization_kernel(energy_density, threshold, dim) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
```

## 6. Test Strategy (100% Coverage)

### Unit Tests (`hadronization_kernel`)
1.  **Zero Input:** Empty `energy_density` array. Expect empty result vector.
2.  **Below Threshold:** `energy_density` values all below `threshold`. Expect empty result vector.
3.  **Above Threshold:** Single value > threshold. Expect 1 vector. Verify magnitude $|\vec{p}| \propto E$.
4.  **Mixed Input:** Mixed values (some <, some > threshold). Expect correct number of vectors.
5.  **Conservation Check:** Sum of norms of output vectors $\approx \sum (E_i)$ for active indices. 
6.  **Dimension Check:** Verify output vectors have requested dimension `dim`.

### Wrapper Tests (`hadronization`)
7.  **Success Propagation:** Valid high-energy input. Verify `PropagatingEffect` contains vector list.
8.  **Error Propagation:** (If applicable, e.g., invalid `dim` = 0). Verify `PropagatingEffect` contains error.
