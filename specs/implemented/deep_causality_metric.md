# Specification: `deep_causality_metric` Crate

**Version**: 0.1.0  
**Status**: Draft  
**Date**: 2025-12-25

---

## 1. Executive Summary

This specification defines the `deep_causality_metric` crate, a foundational crate providing metric signature types for Clifford algebras, Riemannian geometry, and physics sign conventions.

### Key Goals

1. **Single Source of Truth**: Consolidate all metric signature logic into one crate
2. **Zero Dependencies**: Serve as a foundational leaf crate in the dependency graph
3. **Type-Safe Conventions**: Provide compile-time enforcement of physics sign conventions
4. **Cross-Crate Integration**: Enable consistent metric handling across multivector, tensor, topology, and physics crates

---

## 2. Motivation

### Current State

The `Metric` type currently resides in `deep_causality_multivector`:

```
deep_causality_multivector/src/types/metric/mod.rs
```

This creates architectural issues:

| Problem | Impact |
|---------|--------|
| `Metric` is needed by multiple crates | Creates unnecessary coupling |
| Physics conventions (East/West Coast) don't belong in multivector | Blurs responsibility boundaries |
| `deep_causality_topology::ReggeGeometry` imports from multivector | Topology shouldn't need Clifford algebra for metrics |
| `deep_causality_tensor` could use metrics for geometric tensors | Currently not possible without multivector |

### Proposed Dependency Graph

```
                  deep_causality_metric (NEW)
                 /       |       |       \
                ↓        ↓       ↓        ↓
         multivector  tensor  topology  physics
              ↑                   ↑
              |_________↓________|
                      physics
```

---

## 3. Crate Structure

### 3.1 File Layout

```
deep_causality_metric/
├── Cargo.toml
├── README.md
├── LICENSE
├── BUILD.bazel
├── src/
│   ├── lib.rs                      # Public API
│   ├── errors.rs                   # MetricError type
│   ├── types/
│   │   ├── mod.rs
│   │   └── metric.rs               # Core Metric enum
│   ├── conventions/
│   │   ├── mod.rs
│   │   ├── traits.rs               # LorentzianMetric trait
│   │   ├── east_coast.rs           # EastCoastMetric newtype
│   │   ├── west_coast.rs           # WestCoastMetric newtype
│   │   └── aliases.rs              # Domain-specific type aliases
│   └── ops/
│       ├── mod.rs
│       └── convert.rs              # Conversion operations
└── tests/
    ├── metric_tests.rs
    ├── convention_tests.rs
    └── integration_tests.rs
```

### 3.2 Cargo.toml

```toml
[package]
name = "deep_causality_metric"
version = "0.1.0"
edition = "2021"
rust-version = "1.80"
license = "MIT"
repository = "https://github.com/deepcausality/deep_causality.rs"
authors = ["Marvin Hansen <marvin.hansen@gmail.com>"]
description = "Metric signature types and sign conventions for Clifford algebras and physics"
documentation = "https://docs.rs/deep_causality_metric"
categories = ["mathematics", "science"]
keywords = ["metric", "clifford", "spacetime", "signature", "physics"]
exclude = [
    "*.bazel", "*/*.bazel", "*.bazel.*",
    "BUILD", "BUILD.bazel", "MODULE.bazel",
    ".bazelignore", ".bazelrc", "tests/**/*",
]

[features]
default = ["std"]
std = ["alloc"]
alloc = []

# This crate has ZERO external dependencies
# It is a foundational leaf crate
```

---

## 4. Core Types

### 4.1 `Metric` Enum

The core type representing Clifford algebra signatures Cl(p, q, r):

```rust
/// Defines the metric signature of a Clifford Algebra Cl(p, q, r).
///
/// - `p`: Number of basis vectors with e_i² = +1
/// - `q`: Number of basis vectors with e_i² = -1
/// - `r`: Number of degenerate basis vectors with e_i² = 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    /// All basis vectors square to +1. Signature: (N, 0, 0)
    Euclidean(usize),

    /// All basis vectors square to -1. Signature: (0, N, 0)
    NonEuclidean(usize),

    /// West Coast Minkowski: e₀² = +1, others = -1. Signature: (1, N-1, 0)
    Minkowski(usize),

    /// Projective Geometric Algebra: e₀² = 0, others = +1. Signature: (N-1, 0, 1)
    PGA(usize),

    /// Explicit generic signature Cl(p, q, r)
    Generic { p: usize, q: usize, r: usize },

    /// Arbitrary signature via bitmasks (up to 64 dimensions)
    Custom { dim: usize, neg_mask: u64, zero_mask: u64 },
}
```

### 4.2 `MetricError` Enum

```rust
/// Errors that can occur during metric operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricError {
    /// Sign convention mismatch (e.g., expected East Coast, got West Coast)
    SignConventionMismatch(String),
    /// Invalid dimension (e.g., zero or exceeds bitmask capacity)
    InvalidDimension(String),
    /// Metric validation failed
    ValidationFailed(String),
    /// Conversion not possible
    ConversionError(String),
}
```

### 4.3 `LorentzianMetric` Trait

```rust
/// Trait for convention-specific Lorentzian metric wrappers.
///
/// Implemented by `EastCoastMetric` and `WestCoastMetric`.
pub trait LorentzianMetric: Clone + Copy + Debug + PartialEq {
    /// Access the underlying `Metric`
    fn as_metric(&self) -> &Metric;
    
    /// Consume and return the underlying `Metric`
    fn into_metric(self) -> Metric;
    
    /// Standard 4D Minkowski spacetime for this convention
    fn minkowski_4d() -> Self;
    
    /// Standard 3D Minkowski spacetime for this convention
    fn minkowski_3d() -> Self;
    
    /// Returns the sign of the time component: -1 for East, +1 for West
    fn time_sign(&self) -> i32;
    
    /// Returns the sign of a spatial component (index 1)
    fn space_sign(&self) -> i32;
    
    /// Returns true if (-+++) convention
    fn is_east_coast(&self) -> bool;
    
    /// Returns true if (+---) convention
    fn is_west_coast(&self) -> bool;
    
    /// Returns the spacetime dimension
    fn dimension(&self) -> usize;
    
    /// Returns the (p, q, r) signature tuple
    fn signature(&self) -> (usize, usize, usize);
}
```

### 4.4 Convention Newtypes

#### EastCoastMetric

```rust
/// A Lorentzian metric in East Coast (-+++) convention.
///
/// Also known as: MTW, Misner-Thorne-Wheeler, "mostly plus"
///
/// Properties:
/// - e₀² = -1 (time is negative)
/// - e_i² = +1 for i > 0 (space is positive)
/// - Timelike vectors: g(V,V) < 0
/// - 4-velocity: u·u = -c²
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EastCoastMetric(Metric);

impl EastCoastMetric {
    pub const MINKOWSKI_4D: Self = Self(Metric::Custom { 
        dim: 4, neg_mask: 0b0001, zero_mask: 0 
    });
    pub const MINKOWSKI_3D: Self = Self(Metric::Custom { 
        dim: 3, neg_mask: 0b0001, zero_mask: 0 
    });
    
    pub fn new(metric: Metric) -> Result<Self, MetricError>;
    pub fn from_west_coast(metric: Metric) -> Result<Self, MetricError>;
    pub fn new_nd(dim: usize) -> Self;
    pub fn inner(&self) -> &Metric;
}
```

#### WestCoastMetric

```rust
/// A Lorentzian metric in West Coast (+---) convention.
///
/// Also known as: Weinberg, Particle Physics, "mostly minus"
///
/// Properties:
/// - e₀² = +1 (time is positive)
/// - e_i² = -1 for i > 0 (space is negative)
/// - Timelike vectors: g(V,V) > 0
/// - 4-velocity: u·u = +c²
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WestCoastMetric(Metric);

impl WestCoastMetric {
    pub const MINKOWSKI_4D: Self = Self(Metric::Minkowski(4));
    pub const MINKOWSKI_3D: Self = Self(Metric::Minkowski(3));
    
    pub fn new(metric: Metric) -> Result<Self, MetricError>;
    pub fn from_east_coast(metric: Metric) -> Result<Self, MetricError>;
    pub fn new_nd(dim: usize) -> Self;
    pub fn inner(&self) -> &Metric;
}
```

### 4.5 Domain-Specific Type Aliases

```rust
/// For General Relativity modules: East Coast (-+++)
pub type RelativityMetric = EastCoastMetric;
pub const RELATIVITY_MINKOWSKI_4D: RelativityMetric = EastCoastMetric::MINKOWSKI_4D;

/// For Particle Physics modules: West Coast (+---)
pub type ParticleMetric = WestCoastMetric;
pub const PARTICLE_MINKOWSKI_4D: ParticleMetric = WestCoastMetric::MINKOWSKI_4D;

/// Default for crate (configurable): East Coast
pub type PhysicsMetric = RelativityMetric;
pub const MINKOWSKI_4D: PhysicsMetric = RELATIVITY_MINKOWSKI_4D;
```

---

## 5. Core Operations

### 5.1 Metric Methods

```rust
impl Metric {
    /// Total dimension: N = p + q + r
    pub fn dimension(&self) -> usize;
    
    /// Sign of e_i²: returns +1, -1, or 0
    pub fn sign_of_sq(&self, i: usize) -> i32;
    
    /// Signature tuple (p, q, r)
    pub fn signature(&self) -> (usize, usize, usize);
    
    /// Flip time and space signs (convert between conventions)
    pub fn flip_time_space(&self) -> Self;
    
    /// Tensor product of two metrics (for Monad bind)
    pub fn tensor_product(&self, other: &Self) -> Self;
    
    /// Check if two metrics are compatible for operations
    pub fn is_compatible(&self, other: &Self) -> bool;
    
    /// Normalize to Generic form for comparison
    pub fn to_generic(&self) -> Self;
}
```

### 5.2 Conversion Operations

```rust
impl Metric {
    /// Create from (p, q, r) signature
    pub fn from_signature(p: usize, q: usize, r: usize) -> Self;
    
    /// Create Custom from explicit signs array
    pub fn from_signs(signs: &[i32]) -> Result<Self, MetricError>;
    
    /// Extract signs as vector
    pub fn to_signs(&self) -> Vec<i32>;
}
```

---

## 6. Integration with Existing Crates

### 6.1 Integration with `deep_causality_multivector`

#### Current Usage
`CausalMultiVector` stores metric for geometric product computation:

```rust
// Current (in multivector)
pub struct CausalMultiVector<T> {
    pub data: Vec<T>,
    metric: Metric,  // <-- Currently defined in multivector
}
```

#### Migration
After migration, multivector imports from metric crate:

```rust
// New (in multivector)
use deep_causality_metric::Metric;

// Re-export for backwards compatibility
pub use deep_causality_metric::Metric;
```

### 6.2 Integration with `deep_causality_tensor`

#### Current State
`CausalTensor` has no metric awareness:

```rust
pub struct CausalTensor<T> {
    data: Vec<T>,
    shape: Vec<usize>,
    strides: Vec<usize>,
    // No metric field
}
```

#### Proposed Enhancement
Add optional metric for Riemannian tensor operations:

```rust
use deep_causality_metric::Metric;

pub struct MetricTensor<T> {
    tensor: CausalTensor<T>,
    metric: Option<Metric>,
}

impl<T> MetricTensor<T> {
    /// Raises/lowers indices using the metric
    pub fn raise_index(&self, idx: usize) -> Result<Self, TensorError>;
    pub fn lower_index(&self, idx: usize) -> Result<Self, TensorError>;
    
    /// Trace using metric (g^{μν} T_{μν})
    pub fn metric_trace(&self) -> Result<T, TensorError>;
    
    /// Inner product using metric
    pub fn metric_inner(&self, other: &Self) -> Result<T, TensorError>;
}
```

**Note**: This enhancement is OPTIONAL and not part of initial release.

### 6.3 Integration with `deep_causality_topology`

#### Current Usage
`ReggeGeometry` already imports `Metric`:

```rust
// Current (in topology)
use deep_causality_multivector::Metric;  // <-- Imports from multivector

pub struct ReggeGeometry {
    edge_lengths: CausalTensor<f64>,
}

impl ReggeGeometry {
    /// Returns metric signature at a simplex
    pub fn metric_at(&self, complex: &SimplicialComplex, grade: usize, index: usize) -> Metric;
}
```

#### Migration
Change import source:

```rust
// New (in topology)
use deep_causality_metric::Metric;  // <-- Import from metric crate directly
```

#### Proposed Enhancement
Add signature-aware manifold operations:

```rust
use deep_causality_metric::{Metric, LorentzianMetric, PhysicsMetric};

impl ReggeGeometry {
    /// Create with specified signature convention
    pub fn with_signature<M: LorentzianMetric>(
        edge_lengths: CausalTensor<f64>,
        signature: M,
    ) -> Self;
    
    /// Compute local signature from edge lengths (Cayley-Menger)
    pub fn compute_local_signature(&self, simplex_idx: usize) -> Metric;
    
    /// Validate that all local signatures match expected convention
    pub fn validate_signature<M: LorentzianMetric>(&self) -> Result<(), TopologyError>;
}

impl<T> Manifold<T> {
    /// Create manifold with explicit spacetime signature
    pub fn with_lorentzian_metric<M: LorentzianMetric>(
        complex: SimplicialComplex,
        data: CausalTensor<T>,
        signature: M,
    ) -> Result<Self, TopologyError>;
    
    /// Get the spacetime signature
    pub fn spacetime_signature(&self) -> Option<&Metric>;
}
```

---

## 7. Public API Summary

### Exported Types

| Type | Description |
|------|-------------|
| `Metric` | Core signature enum Cl(p, q, r) |
| `MetricError` | Error type for metric operations |
| `LorentzianMetric` | Trait for convention wrappers |
| `EastCoastMetric` | (-+++) convention newtype |
| `WestCoastMetric` | (+---) convention newtype |

### Exported Type Aliases

| Alias | Target | Convention |
|-------|--------|------------|
| `RelativityMetric` | `EastCoastMetric` | (-+++) |
| `ParticleMetric` | `WestCoastMetric` | (+---) |
| `PhysicsMetric` | `RelativityMetric` | Default |

### Exported Constants

| Constant | Type | Value |
|----------|------|-------|
| `MINKOWSKI_4D` | `PhysicsMetric` | 4D Minkowski in default convention |
| `RELATIVITY_MINKOWSKI_4D` | `RelativityMetric` | 4D Minkowski (-+++) |
| `PARTICLE_MINKOWSKI_4D` | `ParticleMetric` | 4D Minkowski (+---) |

---

## 8. Test Plan

### Unit Tests

| Test | Description |
|------|-------------|
| `test_metric_dimension` | Verify dimension() for all variants |
| `test_sign_of_sq` | Verify sign_of_sq() returns correct values |
| `test_signature` | Verify signature() tuple |
| `test_flip_time_space` | Verify convention flip |
| `test_flip_roundtrip` | flip(flip(m)) == m |
| `test_tensor_product` | Verify metric tensor product |
| `test_east_coast_validation` | Reject wrong convention |
| `test_west_coast_validation` | Reject wrong convention |
| `test_east_from_west` | Convert West → East |
| `test_west_from_east` | Convert East → West |

### Integration Tests

| Test | Description |
|------|-------------|
| `test_multivector_import` | Verify re-export works |
| `test_topology_import` | Verify ReggeGeometry compiles |
| `test_physics_conventions` | Verify physics aliases work |

---

## 9. Migration Checklist

### Phase 1: Create Crate
- [ ] Create `deep_causality_metric/` directory
- [ ] Write `Cargo.toml`
- [ ] Move `Metric` from multivector with enhancements
- [ ] Add `MetricError`
- [ ] Add convention newtypes and trait
- [ ] Add type aliases
- [ ] Write tests
- [ ] Write README.md

### Phase 2: Update Multivector
- [ ] Add dependency on `deep_causality_metric`
- [ ] Change `pub use` to re-export from metric crate
- [ ] Delete `src/types/metric/mod.rs`
- [ ] Update tests

### Phase 3: Update Topology
- [ ] Add dependency on `deep_causality_metric`
- [ ] Change import in `ReggeGeometry`
- [ ] Run tests

### Phase 4: Update Physics
- [ ] Add dependency on `deep_causality_metric`
- [ ] Import conventions from metric crate
- [ ] Update kernel signatures
- [ ] Run tests

---

## 10. Appendix: Mathematical Background

### Clifford Algebra Signature

A Clifford algebra Cl(p, q, r) over ℝ^n where n = p + q + r is defined by:

$$e_i \cdot e_j + e_j \cdot e_i = 2g_{ij}$$

where the metric tensor g has:
- p eigenvalues of +1
- q eigenvalues of -1
- r eigenvalues of 0 (degenerate)

### Minkowski Space Conventions

| Convention | Signature | g_{μν} | Used By |
|------------|-----------|--------|---------|
| East Coast | (-+++) | diag(-1,1,1,1) | MTW, GR textbooks |
| West Coast | (+---) | diag(1,-1,-1,-1) | Weinberg, Particle physics |

### Conversion Formula

For any tensor T with indices:

$$T'^{\mu} = g'^{\mu\nu} g_{\nu\rho} T^{\rho}$$

Sign flip affects the 00 component differently than spatial components.
