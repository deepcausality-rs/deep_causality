# Specification: `deep_causality_physics` Crate Update

**Version**: 0.3.0 (Breaking Changes)  
**Status**: Draft  
**Date**: 2025-12-25

---

## 1. Executive Summary

This specification defines updates to the `deep_causality_physics` crate to address:

1. **Sign Convention System**: Adopt conventions from new `deep_causality_metric` crate
2. **GR-MHD Fix**: Refactor `relativistic_current_kernel` to accept `Manifold` for proper covariant divergence
3. **QCD Hadronization**: Replace simplified LPHD model with Lund String Fragmentation

---

## 2. Breaking Changes

> [!CAUTION]
> This is a **breaking release**. The physics crate was released ~1 week ago with minimal adoption, making this the optimal time for breaking changes.

| Change | Impact | Migration |
|--------|--------|-----------|
| `relativistic_current_kernel` signature | High | Update call sites with Manifold |
| `hadronization_kernel` deprecated | Medium | Use `lund_string_fragmentation_kernel` |
| Metric types now from metric crate | Low | Import paths unchanged (re-exported) |

---

## 3. Dependency Changes

### 3.1 Add New Dependency

```toml
[dependencies.deep_causality_metric]
path = "../deep_causality_metric"
version = "0.1.0"
```

### 3.2 Updated Cargo.toml

```toml
[package]
name = "deep_causality_physics"
version = "0.3.0"  # Bumped for breaking changes
# ... rest unchanged

[features]
default = ["std"]
std = ["alloc", "deep_causality_core/std", "deep_causality_num/std"]
alloc = []

[dependencies.deep_causality_metric]
path = "../deep_causality_metric"
version = "0.1.0"

[dependencies.rand]
version = "0.8"
optional = true

[dependencies.rand_distr]
version = "0.4"
optional = true

[features]
# ... existing features
hadronization = ["rand", "rand_distr"]  # Optional QCD support
```

---

## 4. Re-exports from Metric Crate

### 4.1 lib.rs Updates

```rust
// Re-export metric types and conventions
pub use deep_causality_metric::{
    // Core types
    Metric, 
    MetricError,
    
    // Convention trait and wrappers
    LorentzianMetric,
    EastCoastMetric, 
    WestCoastMetric,
    
    // Domain-specific aliases
    PhysicsMetric, 
    RelativityMetric, 
    ParticleMetric,
    
    // Constants
    MINKOWSKI_4D, 
    RELATIVITY_MINKOWSKI_4D, 
    PARTICLE_MINKOWSKI_4D,
};
```

---

## 5. GR-MHD Module Updates

### 5.1 Problem Statement

Current `relativistic_current_kernel`:
- Takes `CausalTensor<f64>` for EM tensor
- Cannot compute covariant divergence ∇_ν F^{μν}
- Returns error: "Cannot compute current J = div F from local tensor F without derivative operator context"

### 5.2 Solution

Accept `Manifold<f64>` which provides differential operators.

### 5.3 New Signature

```rust
use crate::{PhysicsMetric, PhysicsError};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

/// Calculates relativistic current density J^μ via covariant divergence.
///
/// # Physical Model
///
/// Computes the source current from Maxwell's equations:
/// $$ J^\mu = \nabla_\nu F^{\mu\nu} $$
///
/// Using differential forms:
/// $$ J = \delta F = \star d \star F $$
///
/// # Sign Convention
///
/// Uses `PhysicsMetric` (currently East Coast -+++).
///
/// # Arguments
/// * `em_manifold` - Manifold with EM 2-form F on 2-simplices
/// * `spacetime_metric` - Spacetime signature for Hodge star
///
/// # Returns
/// Current density 1-form J
///
/// # Errors
/// - `DimensionMismatch`: Manifold dimension < 4
/// - `CalculationError`: Missing Hodge star operators
pub fn relativistic_current_kernel(
    em_manifold: &Manifold<f64>,
    spacetime_metric: &PhysicsMetric,
) -> Result<CausalTensor<f64>, PhysicsError>;
```

### 5.4 Implementation

```rust
pub fn relativistic_current_kernel(
    em_manifold: &Manifold<f64>,
    spacetime_metric: &PhysicsMetric,
) -> Result<CausalTensor<f64>, PhysicsError> {
    let complex = em_manifold.complex();
    let skeletons = complex.skeletons();
    
    // 1. Validate dimensions
    if skeletons.len() < 3 {
        return Err(PhysicsError::DimensionMismatch(
            "Requires at least 2-simplices for EM 2-form".into()
        ));
    }
    
    if spacetime_metric.dimension() < 4 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Spacetime needs 4D, got {}D", spacetime_metric.dimension()
        )));
    }
    
    // 2. Extract F as 2-form
    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();
    let f_slice = &em_manifold.data().as_slice()[n0 + n1..n0 + n1 + n2];
    
    // 3. Compute J = ★d★F (codifferential)
    let hodge_ops = complex.hodge_star_operators();
    let coboundary_ops = complex.coboundary_operators();
    
    if hodge_ops.len() <= 3 || coboundary_ops.len() <= 2 {
        return Err(PhysicsError::CalculationError(
            "Missing differential operators".into()
        ));
    }
    
    // ★F: 2-form → 2-form (in 4D)
    let star_f = apply_csr_f64(&hodge_ops[2], f_slice);
    
    // d(★F): 2-form → 3-form
    let d_star_f = apply_csr_i8_f64(&coboundary_ops[2], &star_f);
    
    // ★(d★F): 3-form → 1-form
    let j_data = apply_csr_f64(&hodge_ops[3], &d_star_f);
    
    CausalTensor::new(j_data.clone(), vec![j_data.len()])
        .map_err(PhysicsError::from)
}
```

---

## 6. Nuclear Module Updates

### 6.1 New Types

#### LundParameters

```rust
/// Configuration for Lund String Fragmentation.
#[derive(Debug, Clone)]
pub struct LundParameters {
    /// String tension κ ≈ 1 GeV/fm
    pub kappa: f64,
    /// Lund function parameter 'a' (shape)
    pub lund_a: f64,
    /// Lund function parameter 'b' (mass dependence)
    pub lund_b: f64,
    /// Transverse momentum width σ_pT
    pub sigma_pt: f64,
    /// Strange quark suppression s/u
    pub strange_suppression: f64,
    /// Diquark suppression qq/q
    pub diquark_suppression: f64,
    /// Minimum mass to continue fragmentation
    pub min_invariant_mass: f64,
}

impl Default for LundParameters {
    fn default() -> Self {
        Self {
            kappa: 1.0,
            lund_a: 0.68,
            lund_b: 0.98,
            sigma_pt: 0.36,
            strange_suppression: 0.30,
            diquark_suppression: 0.10,
            min_invariant_mass: 0.5,
        }
    }
}
```

#### FourMomentum

```rust
/// Lorentz 4-momentum (E, px, py, pz).
#[derive(Debug, Clone, Copy, Default)]
pub struct FourMomentum {
    pub e: f64,
    pub px: f64,
    pub py: f64,
    pub pz: f64,
}

impl FourMomentum {
    pub fn new(e: f64, px: f64, py: f64, pz: f64) -> Self;
    pub fn invariant_mass_squared(&self) -> f64;  // E² - |p|²
    pub fn invariant_mass(&self) -> f64;
    pub fn momentum_magnitude(&self) -> f64;
    pub fn rapidity(&self) -> f64;
}

impl Add for FourMomentum { ... }
impl Sub for FourMomentum { ... }
```

#### Hadron

```rust
/// Produced hadron from fragmentation.
#[derive(Debug, Clone)]
pub struct Hadron {
    /// PDG particle ID
    pub pdg_id: i32,
    /// 4-momentum in lab frame
    pub momentum: FourMomentum,
}

impl Hadron {
    pub fn new(pdg_id: i32, momentum: FourMomentum) -> Self;
    pub fn mass(&self) -> f64;
}
```

### 6.2 New Kernel

```rust
/// QCD-inspired hadronization using Lund String Fragmentation.
///
/// # Physical Model
///
/// Models QCD color flux tube as relativistic string with tension κ.
/// String breaks via q-qbar pair creation when E > threshold.
///
/// # Arguments
/// * `string_endpoints` - (quark, antiquark) 4-momentum pairs
/// * `params` - Lund fragmentation parameters
/// * `rng` - Random number generator
///
/// # Returns
/// List of produced hadrons
///
/// # Feature
/// Requires `hadronization` feature flag.
#[cfg(feature = "hadronization")]
pub fn lund_string_fragmentation_kernel<R: rand::Rng>(
    string_endpoints: &[(FourMomentum, FourMomentum)],
    params: &LundParameters,
    rng: &mut R,
) -> Result<Vec<Hadron>, PhysicsError>;
```

### 6.3 Deprecation

```rust
#[deprecated(
    since = "0.3.0",
    note = "Use `lund_string_fragmentation_kernel` for QCD-accurate hadronization"
)]
pub fn hadronization_kernel(
    energy_density: &[EnergyDensity],
    threshold: f64,
    dim: usize,
) -> Result<Vec<PhysicalVector>, PhysicsError>;
```

---

## 7. Error Type Updates

### 7.1 New Error Variant

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicsError {
    // Existing variants...
    DimensionMismatch(String),
    CalculationError(String),
    PhysicalInvariantBroken(String),
    NumericalInstability(String),
    
    // NEW: Metric convention error (wraps MetricError)
    MetricConventionError(deep_causality_metric::MetricError),
}

impl From<deep_causality_metric::MetricError> for PhysicsError {
    fn from(e: MetricError) -> Self {
        PhysicsError::MetricConventionError(e)
    }
}
```

---

## 8. Module Structure Updates

### 8.1 File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/lib.rs` | Modify | Add metric re-exports |
| `src/mhd/grmhd.rs` | Modify | Update `relativistic_current_kernel` |
| `src/nuclear/physics.rs` | Modify | Add Lund types, deprecate old kernel |
| `src/nuclear/quantities.rs` | Modify | Add `FourMomentum`, `Hadron` |
| `src/errors.rs` | Modify | Add `MetricConventionError` |

### 8.2 No Changes Needed

The following files need NO changes (they don't directly use Metric):
- All thermodynamics kernels
- All astro kernels
- All electromagnetism kernels
- All fluids kernels
- All wave kernels
- All dynamics kernels

---

## 9. Test Updates

### 9.1 New Tests for GR-MHD

```rust
#[test]
fn test_relativistic_current_requires_manifold() {
    // Verify proper manifold is required
}

#[test]
fn test_relativistic_current_dimension_check() {
    // Verify 4D requirement
}

#[test]
fn test_relativistic_current_produces_1form() {
    // Verify output shape
}
```

### 9.2 New Tests for Lund

```rust
#[test]
fn test_lund_default_params() {
    let params = LundParameters::default();
    assert!((params.kappa - 1.0).abs() < 0.01);
}

#[test]
fn test_four_momentum_invariant_mass() {
    let p = FourMomentum::new(5.0, 3.0, 0.0, 4.0);
    assert!((p.invariant_mass_squared() - 0.0).abs() < 0.01); // Massless
}

#[test]
#[cfg(feature = "hadronization")]
fn test_lund_momentum_conservation() {
    // Total output momentum ≈ input momentum
}
```

---

## 10. Migration Guide

### For Users of `relativistic_current_kernel`

**Before (v0.2)**:
```rust
// This always returned Err because it couldn't compute divergence
let result = relativistic_current_kernel(&em_tensor, &metric);
```

**After (v0.3)**:
```rust
use deep_causality_physics::{PhysicsMetric, MINKOWSKI_4D};
use deep_causality_topology::Manifold;

// Create manifold with EM field data
let manifold = Manifold::new(complex, em_data, 0)?;

// Now works with proper differential operators
let current = relativistic_current_kernel(&manifold, &MINKOWSKI_4D)?;
```

### For Users of `hadronization_kernel`

**Before (v0.2)**:
```rust
let particles = hadronization_kernel(&energy_densities, threshold, 3)?;
```

**After (v0.3)**:
```rust
use deep_causality_physics::{
    LundParameters, FourMomentum, lund_string_fragmentation_kernel
};

let params = LundParameters::default();
let endpoints = vec![
    (FourMomentum::new(50.0, 0.0, 0.0, 50.0), 
     FourMomentum::new(50.0, 0.0, 0.0, -50.0)),
];
let mut rng = rand::thread_rng();
let hadrons = lund_string_fragmentation_kernel(&endpoints, &params, &mut rng)?;
```

---

## 11. Implementation Checklist

### Phase 1: Dependencies
- [ ] Add `deep_causality_metric` dependency to Cargo.toml
- [ ] Add optional `rand` and `rand_distr` dependencies
- [ ] Add `hadronization` feature flag

### Phase 2: Re-exports
- [ ] Update `src/lib.rs` with metric re-exports
- [ ] Add `MetricConventionError` to `PhysicsError`

### Phase 3: GR-MHD
- [ ] Update `relativistic_current_kernel` signature
- [ ] Implement manifold-based divergence computation
- [ ] Add tests

### Phase 4: Nuclear
- [ ] Add `LundParameters` struct
- [ ] Add `FourMomentum` type
- [ ] Add `Hadron` type
- [ ] Implement `lund_string_fragmentation_kernel`
- [ ] Add `#[deprecated]` to `hadronization_kernel`
- [ ] Add tests

### Phase 5: Verification
- [ ] Run `cargo test -p deep_causality_physics --all-features`
- [ ] Run `cargo clippy -p deep_causality_physics`
- [ ] Update documentation
- [ ] Update CHANGELOG.md

---

## 12. Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.2.0 | 2025-12-18 | Initial release |
| 0.3.0 | TBD | This spec: metric integration, GR-MHD fix, Lund model |
