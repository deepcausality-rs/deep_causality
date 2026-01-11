# Gauge Field Infrastructure: HKT-Enabled Gauge Theory Foundation

* **Product Area:** Deep Causality
* **Crate:** `deep_causality_topology`
* **Status:** Implemented and Reviewed.  
* **Target:** Q1 2026
* **Classification:** Core Infrastructure
* **Owner:** DeepCausality Authors

---

## 1. Executive Summary

This document specifies the **Gauge Field infrastructure** in `deep_causality_topology`. This provides
the foundational types and HKT traits that enable physics theories to be implemented in `deep_causality_physics`.

### 1.1 Separation of Concerns

| Crate                     | Responsibility                                                |
|---------------------------|---------------------------------------------------------------|
| `deep_causality_topology` | GaugeField struct, GaugeGroup trait, HKT witnesses            |
| `deep_causality_physics`  | Theory implementations (QED, GR, QCD) — see gauge_theories.md |

### 1.2 Problem Statement

The current `Manifold<T>` provides single-type HKT (Functor, Monad, CoMonad) but cannot implement:

- ❌ Promonad (needs 3 type params)
- ❌ ParametricMonad (needs 3 type params)
- ❌ RiemannMap (needs 4 type params)
- ❌ Adjunction (needs paired witnesses)

### 1.3 Solution

| New Type                   | HKT Trait(s)              | Purpose                                       |
|----------------------------|---------------------------|-----------------------------------------------|
| `GaugeField<G, A, F>`      | Promonad, ParametricMonad | Current-field coupling, gauge transformations |
| `CurvatureTensor<A,B,C,D>` | RiemannMap                | Curvature contraction, scattering             |
| `d ⊣ ∂` witness pair       | Adjunction                | Stokes theorem, conservation                  |

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  deep_causality_topology                                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Types (src/types/)                                                     │
│  ┌────────────────┐  ┌──────────────────┐  ┌───────────────────┐        │
│  │  GaugeField    │  │  CurvatureTensor │  │ DifferentialForm  │        │
│  │  <G, A, F>     │  │  <A, B, C, D>    │  │      <T>          │        │
│  └───────┬────────┘  └────────┬─────────┘  └─────────┬─────────┘        │
│          │                    │                      │                  │
│  Extensions (src/extensions/)                                           │
│  ┌────────────────┐  ┌──────────────────┐  ┌───────────────────┐        │
│  │GaugeFieldWit.  │  │CurvatureTensorW. │  │ StokesAdjunction  │        │
│  │ • Promonad     │  │ • RiemannMap     │  │ • Adjunction d⊣∂  │        │
│  │ • Parametric   │  │                  │  │                   │        │
│  │   Monad        │  │                  │  │                   │        │
│  └────────────────┘  └──────────────────┘  └───────────────────┘        │
│                                                                         │
│  Groups (src/types/gauge_field/groups/)                                 │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌────────┐ ┌─────────────┐ ┌───────────────┐   │
│  │ U1  │ │ SU2 │ │ SU3 │ │Lorentz │ │ Electroweak │ │StandardModel  │   │
│  └─────┘ └─────┘ └─────┘ └────────┘ └─────────────┘ └───────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. File Structure

### 3.1 New Files to Create

| File Path                                        | Description                              |
|--------------------------------------------------|------------------------------------------|
| `src/types/gauge_field/mod.rs`                   | GaugeField struct, constructors, getters |
| `src/types/gauge_field/group.rs`                 | GaugeGroup trait                         |
| `src/types/gauge_field/groups/mod.rs`            | Re-exports all gauge groups              |
| `src/types/gauge_field/groups/u1.rs`             | U(1) - electromagnetism                  |
| `src/types/gauge_field/groups/su2.rs`            | SU(2) - weak isospin                     |
| `src/types/gauge_field/groups/su3.rs`            | SU(3) - color charge                     |
| `src/types/gauge_field/groups/lorentz.rs`        | SO(3,1) - Lorentz group                  |
| `src/types/gauge_field/groups/electroweak.rs`    | SU(2)×U(1) - electroweak                 |
| `src/types/gauge_field/groups/standard_model.rs` | SU(3)×SU(2)×U(1) - Standard Model        |
| `src/types/curvature_tensor/mod.rs`              | CurvatureTensor struct                   |
| `src/types/differential_form/mod.rs`             | DifferentialForm type                    |
| `src/extensions/hkt_gauge_field.rs`              | Promonad, ParametricMonad impls          |
| `src/extensions/hkt_curvature.rs`                | RiemannMap impl                          |
| `src/extensions/adjunction_stokes.rs`            | Stokes adjunction (d ⊣ ∂)                |

### 3.2 Files to Modify

| File Path               | Changes                                    |
|-------------------------|--------------------------------------------|
| `src/types/mod.rs`      | Add `gauge_field`, `curvature_tensor` mods |
| `src/extensions/mod.rs` | Add HKT extension modules                  |
| `src/lib.rs`            | Re-export new types                        |

### 3.3 Updated lib.rs Exports

```rust
// Location: deep_causality_topology/src/lib.rs (additions)

// Gauge Field types
pub use crate::types::gauge_field::{GaugeField, GaugeGroup};
pub use crate::types::gauge_field::groups::{
    U1, SU2, SU3, Lorentz, Electroweak, StandardModel,
};

// Curvature types
pub use crate::types::curvature_tensor::{CurvatureTensor, CurvatureSymmetry};
pub use crate::types::differential_form::DifferentialForm;

// HKT extensions
pub use crate::extensions::hkt_gauge_field::GaugeFieldWitness;
pub use crate::extensions::hkt_curvature::CurvatureTensorWitness;
pub use crate::extensions::adjunction_stokes::{
    ExteriorDerivativeWitness, BoundaryWitness, StokesAdjunction, StokesContext,
};
```

---

## 4. Metric Integration

The gauge field infrastructure uses `deep_causality_metric` for metric signature handling.

### 4.1 Key Types from deep_causality_metric

| Type               | Description                                                |
|--------------------|------------------------------------------------------------|
| `Metric`           | Enum: Euclidean, Minkowski, PGA, Generic, Custom           |
| `EastCoastMetric`  | Wrapper for (-+++) convention (GR, MTW)                    |
| `WestCoastMetric`  | Wrapper for (+---) convention (Particle physics, Weinberg) |
| `LorentzianMetric` | Trait for convention-specific operations                   |

### 4.2 Sign Conventions

> [!IMPORTANT]
> **GR vs Particle Physics use opposite sign conventions!**

| Convention | Signature | g_{μν}           | Used By                    |
|------------|-----------|------------------|----------------------------|
| East Coast | (-+++)    | diag(-1,1,1,1)   | MTW, GR textbooks          |
| West Coast | (+---)    | diag(1,-1,-1,-1) | Weinberg, Particle physics |

### 4.3 Metric Usage in Gauge Fields

```rust
use deep_causality_metric::{Metric, EastCoastMetric, WestCoastMetric, LorentzianMetric};

// For GR: use East Coast convention
let gr_metric = EastCoastMetric::minkowski_4d();
assert_eq!(gr_metric.time_sign(), -1);
assert_eq!(gr_metric.space_sign(), 1);

// For QED/QCD: use West Coast convention
let qed_metric = WestCoastMetric::minkowski_4d();
assert_eq!(qed_metric.time_sign(), 1);
assert_eq!(qed_metric.space_sign(), -1);

// Convert between conventions
let flipped = Metric::Minkowski(4).flip_time_space();
```

---

## 5. Technical Specification

### 5.1 GaugeGroup Trait

```rust
// Location: src/types/gauge_field/group.rs

use std::fmt::Debug;
use deep_causality_metric::Metric;

/// Marker trait for gauge groups.
///
/// A gauge group defines the local symmetry of a gauge field theory.
pub trait GaugeGroup: Clone + Debug + Send + Sync + 'static {
    /// Dimension of the Lie algebra (number of generators).
    const LIE_ALGEBRA_DIM: usize;

    /// Whether the group is abelian.
    /// - Abelian: F = dA
    /// - Non-abelian: F = dA + A∧A
    const IS_ABELIAN: bool;

    /// Spacetime dimension (default 4).
    const SPACETIME_DIM: usize = 4;

    /// Human-readable name.
    fn name() -> &'static str;

    /// Default metric for this gauge group.
    /// Override for specific physics conventions.
    fn default_metric() -> Metric {
        Metric::Minkowski(Self::SPACETIME_DIM)
    }
}
```

### 5.2 Gauge Group Implementations

```rust
// Location: src/types/gauge_field/groups/*.rs

use deep_causality_metric::Metric;

#[derive(Clone, Debug, Default)]
pub struct U1;  // Lie dim = 1, abelian

impl GaugeGroup for U1 {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;
    fn name() -> &'static str { "U(1)" }
    // Uses West Coast by default (particle physics)
}

#[derive(Clone, Debug, Default)]
pub struct SU2; // Lie dim = 3, non-abelian

#[derive(Clone, Debug, Default)]
pub struct SU3; // Lie dim = 8, non-abelian

#[derive(Clone, Debug, Default)]
pub struct Lorentz; // Lie dim = 6, non-abelian

impl GaugeGroup for Lorentz {
    const LIE_ALGEBRA_DIM: usize = 6;
    const IS_ABELIAN: bool = false;
    fn name() -> &'static str { "SO(3,1)" }
    // GR convention: East Coast (-+++)
    fn default_metric() -> Metric {
        Metric::from_signature(0, 3, 0) // One time (-), three space (+)
        // Actually: (1, 3, 0) with flip gives East Coast
    }
}

#[derive(Clone, Debug, Default)]
pub struct Electroweak; // Lie dim = 4, non-abelian

#[derive(Clone, Debug, Default)]
pub struct StandardModel; // Lie dim = 12, non-abelian
```

### 5.3 GaugeField<G, A, F> Structure

```rust
// Location: src/types/gauge_field/mod.rs

use crate::Manifold;
use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

/// A gauge field over a base manifold.
///
/// # Type Parameters
/// * `G` - Gauge group (U1, SU2, SU3, Lorentz, etc.)
/// * `A` - Connection (potential) scalar type
/// * `F` - Field strength (curvature) scalar type
#[derive(Debug, Clone)]
pub struct GaugeField<G: GaugeGroup, A, F> {
    /// Base manifold. Private.
    base: Manifold<f64>,

    /// Spacetime metric signature.
    metric: Metric,

    /// Gauge connection (potential).
    /// Shape: [num_points, spacetime_dim, lie_algebra_dim]
    connection: CausalTensor<A>,

    /// Field strength (curvature).
    /// Shape: [num_points, spacetime_dim, spacetime_dim, lie_algebra_dim]
    field_strength: CausalTensor<F>,

    /// Gauge group marker.
    _gauge: PhantomData<G>,
}

// Constructors
impl<G: GaugeGroup, A, F> GaugeField<G, A, F> {
    /// Create with explicit metric.
    pub fn new(
        base: Manifold<f64>,
        metric: Metric,
        connection: CausalTensor<A>,
        field_strength: CausalTensor<F>,
    ) -> Self { ... }

    /// Create with default metric for the gauge group.
    pub fn with_default_metric(
        base: Manifold<f64>,
        connection: CausalTensor<A>,
        field_strength: CausalTensor<F>,
    ) -> Self {
        Self::new(base, G::default_metric(), connection, field_strength)
    }
}

// Getters
impl<G: GaugeGroup, A, F> GaugeField<G, A, F> {
    pub fn base(&self) -> &Manifold<f64>;
    pub fn metric(&self) -> Metric;
    pub fn connection(&self) -> &CausalTensor<A>;
    pub fn field_strength(&self) -> &CausalTensor<F>;
    pub fn gauge_group_name(&self) -> &'static str;
    pub fn lie_algebra_dim(&self) -> usize;
    pub fn is_abelian(&self) -> bool;
    pub fn spacetime_dim(&self) -> usize;

    /// Check if using East Coast convention (-+++).
    pub fn is_east_coast(&self) -> bool {
        self.metric.sign_of_sq(0) == -1
    }

    /// Check if using West Coast convention (+---).
    pub fn is_west_coast(&self) -> bool {
        self.metric.sign_of_sq(0) == 1 && self.metric.sign_of_sq(1) == -1
    }
}
```

### 4.4 CurvatureTensor<A, B, C, D> Structure

```rust
// Location: src/types/curvature_tensor/mod.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurvatureSymmetry {
    Riemann,
    Weyl,
    Ricci,
    None,
}

/// Rank-4 curvature tensor for RiemannMap.
#[derive(Debug, Clone)]
pub struct CurvatureTensor<A, B, C, D> {
    components: CausalTensor<f64>,
    metric: Metric,
    symmetry: CurvatureSymmetry,
    dim: usize,
    _phantom: PhantomData<(A, B, C, D)>,
}

impl<A, B, C, D> CurvatureTensor<A, B, C, D> {
    // Constructors
    pub fn new(components: CausalTensor<f64>, metric: Metric, symmetry: CurvatureSymmetry, dim: usize) -> Self;
    pub fn flat(dim: usize) -> Self;
    pub fn flat_with_metric(dim: usize, metric: Metric) -> Self;

    // Getters
    pub fn components(&self) -> &CausalTensor<f64>;
    pub fn metric(&self) -> Metric;
    pub fn symmetry(&self) -> CurvatureSymmetry;
    pub fn dim(&self) -> usize;
    pub fn ricci_scalar(&self) -> f64;
}
```

---

## 5. HKT Trait Implementations

### 5.1 GaugeFieldWitness (HKT3Unbound)

```rust
// Location: src/extensions/hkt_gauge_field.rs

pub struct GaugeFieldWitness;

impl HKT3Unbound for GaugeFieldWitness {
    type Constraint = NoConstraint;
    type Type<G, A, F> = GaugeField<G, A, F>
    where
        G: Satisfies<NoConstraint> + GaugeGroup,
        A: Satisfies<NoConstraint>,
        F: Satisfies<NoConstraint>;
}
```

#### 5.1.1 Promonad Implementation

```rust
/// Promonad models: Current + Potential → Field Strength
///
/// Physics: ∂_μF^μν = J^ν (Maxwell/Yang-Mills equations)
impl Promonad<GaugeFieldWitness> for GaugeFieldWitness {
    fn merge<J, A, F, Func>(
        current: GaugeField<J, J, J>,
        potential: GaugeField<A, A, A>,
        f: Func,
    ) -> GaugeField<F, F, F>
    where
        Func: FnMut(J, A) -> F,
    { ... }

    fn fuse<J, A, F>(current: J, potential: A) -> GaugeField<J, A, F>
    { ... }
}
```

#### 5.1.2 ParametricMonad Implementation

```rust
/// ParametricMonad models: Gauge transformations S1 → S2 → S3
///
/// Physics: A' = gAg⁻¹ + g∂g⁻¹ (gauge covariance)
impl ParametricMonad<GaugeFieldWitness> for GaugeFieldWitness {
    fn pure<S, A>(value: A) -> GaugeField<S, S, A> { ... }

    fn ibind<S1, S2, S3, A, B, Func>(
        m: GaugeField<S1, S2, A>,
        f: Func,
    ) -> GaugeField<S1, S3, B>
    where
        Func: FnMut(A) -> GaugeField<S2, S3, B>,
    { ... }
}
```

### 5.2 CurvatureTensorWitness (HKT4Unbound)

```rust
// Location: src/extensions/hkt_curvature.rs

pub struct CurvatureTensorWitness;

impl HKT4Unbound for CurvatureTensorWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C, D> = CurvatureTensor<A, B, C, D>;
}

/// RiemannMap: Curvature tensor operations
impl RiemannMap<CurvatureTensorWitness> for CurvatureTensorWitness {
    /// R(u,v)w computes parallel transport holonomy
    fn curvature<A, B, C, D>(
        tensor: CurvatureTensor<A, B, C, D>,
        u: A, v: B, w: C,
    ) -> D { ... }

    /// S-matrix scattering: (A, B) → (C, D)
    fn scatter<A, B, C, D>(
        interaction: CurvatureTensor<A, B, C, D>,
        in_1: A, in_2: B,
    ) -> (C, D) { ... }
}
```

### 5.3 StokesAdjunction (Adjunction d ⊣ ∂)

```rust
// Location: src/extensions/adjunction_stokes.rs

pub struct ExteriorDerivativeWitness; // d: Ω^k → Ω^(k+1)
pub struct BoundaryWitness;           // ∂: C_k → C_(k-1)

pub struct StokesContext {
    complex: SimplicialComplex,
}

/// Stokes theorem: ⟨dω, C⟩ = ⟨ω, ∂C⟩
impl Adjunction<ExteriorDerivativeWitness, BoundaryWitness, StokesContext>
for StokesAdjunction
{
    fn unit<A>(...) -> Chain<DifferentialForm<A>> { ... }
    fn counit<B>(...) -> B { ... }
    fn left_adjunct<A, B, Func>(...) -> Chain<B> { ... }
    fn right_adjunct<A, B, Func>(...) -> B { ... }
}
```

---

## 6. Verification Plan

| Component             | Tests                                   |
|-----------------------|-----------------------------------------|
| `GaugeField`          | Construction, getters, shape validation |
| `GaugeGroup` impls    | Constants, name() method                |
| `CurvatureTensor`     | Symmetry, flat tensor, contractions     |
| `GaugeFieldWitness`   | Promonad laws, ParametricMonad laws     |
| `CurvatureTensorWit.` | RiemannMap correctness                  |
| `StokesAdjunction`    | Triangle identities                     |

---

## 7. Related Specifications

| Document            | Content                                      |
|---------------------|----------------------------------------------|
| `gauge_theories.md` | QED, GR, QCD implementations (physics crate) |
