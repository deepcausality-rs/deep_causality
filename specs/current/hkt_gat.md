# GAT-Bounded Higher-Kinded Types for Rust: Product Specification

> **Product Area:** Deep Causality | **Crate:** `deep_causality_haft`  
> **Status:** Production Planning | **Target:** Q1 2026  
> **Classification:** Core Infrastructure | **Owner:** DeepCausality Authors

---

## 1. Executive Summary

This document specifies the next-generation **GAT-Bounded HKT** system for the `deep_causality_haft` crate, designed to
enable **monadic composition** across constrained type constructors while maintaining production-grade quality
standards. The system introduces a principled `Satisfies<C>` pattern to encode type constraints at the trait level,
unlocking HKT operations for physics-oriented types like `CausalMultiField`, `SpinorManifold`, and `GaugeManifold`.

### 1.1 Problem Statement

Current bounded traits (`BoundedComonad`, `BoundedAdjunction`) embed **arbitrary, hardcoded bounds** (e.g.,
`Zero + Copy + Clone`) directly in trait method signatures. This creates:

1. **Universality Violation**: Types requiring different bounds cannot implement the same trait
2. **API Fragmentation**: Different types need different trait hierarchies
3. **TensorData Incompatibility**: `CausalMultiField<B, T: TensorData>` cannot implement `HKT` because GATs in the
   current `HKT` trait accept unbounded `T`

### 1.2 Solution Overview

Introduce **Bounded GATs** using the `Satisfies<Constraint>` pattern:

```rust
// The constraint is declared by the HKT implementor, not hardcoded in the trait
pub trait BoundedHKT {
    type Constraint: ?Sized;
    type Type<T>
    where
        T: Satisfies<Self::Constraint>;
}
```

This enables:

- **Unified type system** across constrained and unconstrained types
- **Monadic composition** for physics types with `TensorData` bounds
- **Compile-time constraint verification** without runtime overhead

### 1.3 Key Decisions

> [!IMPORTANT]
> **Migration Strategy: Complete Replacement**

| Decision                      | Rationale                                                              |
|-------------------------------|------------------------------------------------------------------------|
| **No backward compatibility** | Zero external adoption of current bounded traits justifies clean break |
| **No feature flags**          | Simple rip-and-replace; no compile-time separation needed              |
| **No V2 naming**              | Traits retain original names (`BoundedComonad`, `BoundedAdjunction`)   |
| **Legacy traits removed**     | After migration, no traces of old implementations remain               |
| **Default implementations**   | Welcomed wherever sensible to reduce boilerplate                       |

---

## 2. Background & Motivation

### 2.1 Current State Analysis

The `deep_causality_haft` crate currently provides two parallel trait hierarchies:

| Category      | Trait                                 | Bound Location                  |
|---------------|---------------------------------------|---------------------------------|
| **Unbounded** | `HKT`, `Functor`, `Monad`, `CoMonad`  | No bounds on `T`                |
| **Bounded**   | `BoundedComonad`, `BoundedAdjunction` | `Zero + Copy + Clone` hardcoded |

**Active Implementations (from codebase search):**

| Witness Type               | Crate                        | Implements                                                       |
|----------------------------|------------------------------|------------------------------------------------------------------|
| `CausalTensorWitness`      | `deep_causality_tensor`      | `HKT`, `Functor`, `Monad`, `BoundedComonad`, `BoundedAdjunction` |
| `CausalMultiVectorWitness` | `deep_causality_multivector` | `HKT`, `Functor`, `Monad`, `BoundedComonad`, `BoundedAdjunction` |
| `ManifoldWitness`          | `deep_causality_topology`    | `HKT`, `Functor`, `BoundedComonad`                               |
| `TopologyWitness`          | `deep_causality_topology`    | `HKT`, `Functor`, `BoundedComonad`                               |
| `GraphWitness`             | `deep_causality_topology`    | `HKT`, `Functor`, `BoundedComonad`                               |
| `HypergraphWitness`        | `deep_causality_topology`    | `HKT`, `Functor`, `BoundedComonad`                               |
| `PointCloudWitness`        | `deep_causality_topology`    | `HKT`, `Functor`, `BoundedComonad`                               |
| `CsrMatrixWitness`         | `deep_causality_sparse`      | `HKT`, `Functor`, `BoundedComonad`, `BoundedAdjunction`          |
| `ChainWitness`             | `deep_causality_topology`    | `HKT` (used in `BoundedAdjunction`)                              |

### 2.2 The TensorData Problem

From `specs/current/hkt_fields.md`, the core incompatibility:

```rust
// TensorData is a rich constraint
pub trait TensorData: Field + Copy + Default + PartialOrd + Send + Sync + 'static {}

// CausalMultiField requires T: TensorData
pub struct CausalMultiField<B: LinearAlgebraBackend, T: TensorData> {
    ...
}

// HKT requires unconditional GAT
pub trait HKT {
    type Type<T>;  // NO BOUNDS - incompatible with CausalMultiField!
}
```

**Cannot implement:**

```rust
impl<B: LinearAlgebraBackend> HKT for CausalMultiFieldWitness<B> {
    type Type<T> = CausalMultiField<B, T>;  // ERROR: T needs TensorData
}
```

### 2.3 Physics Use Cases (from `specs/current/topo_physics.md`)

The HKT system encodes physical operations:

| Operation       | HKT Abstraction          | Physics Example                        |
|-----------------|--------------------------|----------------------------------------|
| `fmap`          | Transform field values   | Scale electric field                   |
| `extend`        | Apply stencil everywhere | Laplacian → Heat diffusion             |
| `left_adjunct`  | Differentiation          | Field from charge density (∇·E = ρ/ε₀) |
| `right_adjunct` | Integration              | ∫ E·dl (line integral)                 |
| `bind`          | Chain computations       | Multi-stage particle physics           |

**SpinorManifold and GaugeManifold** require `TensorData` bounds but need full HKT trait access for:

- Dirac operator: `D̸ψ = γ^μ ∂_μ ψ`
- Wilson action: `S = β Σ_P (1 - Re Tr U_P)`
- Gauge covariance transformations

---

## 3. Technical Specification

### 3.1 Core Traits

#### 3.1.1 Constraint Marker Trait

```rust
// Location: deep_causality_haft/src/core/constraint.rs

/// Marker trait indicating that type `T` satisfies constraint meta-type `C`.
///
/// This trait is the foundation of the Bounded GAT system. It allows type-level
/// encoding of constraints without hardcoding them into trait signatures.
///
/// # Implementation Pattern
///
/// For each constraint (like `TensorData`), define a marker struct and implement
/// `Satisfies` for all types that meet the constraint:
///
/// ```rust
/// pub struct TensorDataConstraint;
/// impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}
/// ```
pub trait Satisfies<C: ?Sized> {}

/// Marker for types with no constraint (fully polymorphic).
pub struct NoConstraint;

/// Blanket implementation: everything satisfies NoConstraint.
/// This is the default implementation that enables unconstrained HKTs.
impl<T> Satisfies<NoConstraint> for T {}

/// Marker for thread-safe types (Send + Sync).
pub struct ThreadSafeConstraint;

/// Blanket implementation: all Send + Sync types satisfy ThreadSafeConstraint.
impl<T: Send + Sync> Satisfies<ThreadSafeConstraint> for T {}
```

#### 3.1.2 BoundedHKT Trait

```rust
// Location: deep_causality_haft/src/core/hkt_bounded.rs

/// Higher-Kinded Type with declarative constraint.
///
/// This trait allows implementors to specify their required bounds via the
/// `Constraint` associated type, while GAT `Type<T>` only accepts `T` that
/// satisfies the constraint.
///
/// # Category Theory
/// Corresponds to a **Restricted Functor** or an endofunctor on a subcategory
/// of Rust types defined by the constraint.
///
/// # Example: TensorData-Constrained HKT
///
/// ```rust
/// impl<B: LinearAlgebraBackend> BoundedHKT for CausalMultiFieldWitness<B> {
///     type Constraint = TensorDataConstraint;
///     type Type<T> = CausalMultiField<B, T> where T: Satisfies<TensorDataConstraint>;
/// }
/// ```
///
/// # Example: Unconstrained HKT (uses default NoConstraint)
///
/// ```rust
/// impl BoundedHKT for VecWitness {
///     type Constraint = NoConstraint;  // Everything satisfies this
///     type Type<T> = Vec<T> where T: Satisfies<NoConstraint>;
/// }
/// ```
pub trait BoundedHKT {
    /// The meta-type representing the constraint on inner types.
    /// Use `NoConstraint` for unconstrained HKTs (blanket impl covers all T).
    type Constraint: ?Sized;

    /// The type constructor. `T` must satisfy `Self::Constraint`.
    type Type<T>
    where
        T: Satisfies<Self::Constraint>;
}
```

#### 3.1.3 BoundedFunctor Trait

```rust
// Location: deep_causality_haft/src/algebra/functor_bounded.rs

/// Functor for constrained type constructors.
///
/// Maps a function `A -> B` over the structure, preserving the container.
/// Both `A` and `B` must satisfy the HKT's constraint.
///
/// # Laws
/// 1. **Identity**: `fmap(fa, id) == fa`
/// 2. **Composition**: `fmap(fa, f.compose(g)) == fmap(fmap(fa, g), f)`
///
/// # Default Implementation
///
/// Types implementing `BoundedHKT` with iterable data can derive `fmap`
/// via a provided `fmap_default` helper (see extensions module).
pub trait BoundedFunctor<F: BoundedHKT> {
    fn fmap<A, B, Func>(fa: F::Type<A>, f: Func) -> F::Type<B>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> B;
}
```

#### 3.1.4 BoundedApplicative Trait

```rust
// Location: deep_causality_haft/src/algebra/applicative_bounded.rs

/// Applicative functor for constrained type constructors.
///
/// Extends `BoundedFunctor` with the ability to lift values and apply
/// functions within the constrained context.
///
/// # Default Implementation
///
/// `apply` has a default implementation in terms of `bind` when
/// `BoundedMonad` is also implemented:
///
/// ```rust
/// fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
/// where ...
/// {
///     Self::bind(f_ab, |func| Self::fmap(f_a.clone(), func))
/// }
/// ```
pub trait BoundedApplicative<F: BoundedHKT>: BoundedFunctor<F> {
    /// Lift a value into the minimal context.
    fn pure<T>(value: T) -> F::Type<T>
    where
        T: Satisfies<F::Constraint>;

    /// Apply a wrapped function to a wrapped value.
    ///
    /// Default implementation provided when `BoundedMonad` is implemented.
    fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
    where
        A: Satisfies<F::Constraint> + Clone,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> B;
}
```

#### 3.1.5 BoundedMonad Trait

```rust
// Location: deep_causality_haft/src/algebra/monad_bounded.rs

/// Monad for constrained type constructors.
///
/// Provides the `bind` operation for sequencing computations that produce
/// constrained effectful values. This is the key enabler for monadic
/// composition in physics simulations.
///
/// # Laws
/// 1. **Left Identity**: `bind(pure(a), f) == f(a)`
/// 2. **Right Identity**: `bind(m, pure) == m`
/// 3. **Associativity**: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`
///
/// # Default Implementations
///
/// - `join`: Flatten nested structure, default in terms of `bind`
/// - `fmap`: Default in terms of `bind` and `pure`
pub trait BoundedMonad<F: BoundedHKT>: BoundedApplicative<F> {
    /// Bind (flatMap) operation.
    ///
    /// Chains a computation from an effectful value, flattening the result.
    fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> F::Type<B>;

    /// Flatten a nested structure.
    ///
    /// Default implementation: `bind(m_m_a, |x| x)`
    fn join<A>(m_m_a: F::Type<F::Type<A>>) -> F::Type<A>
    where
        A: Satisfies<F::Constraint>,
        F::Type<A>: Satisfies<F::Constraint>,
    {
        Self::bind(m_m_a, |x| x)
    }
}
```

#### 3.1.6 BoundedComonad Trait (Replacement)

```rust
// Location: deep_causality_haft/src/algebra/comonad.rs (REPLACES existing)

/// Comonad for constrained type constructors.
///
/// The dual of `BoundedMonad`, providing `extract` (get focus value) and
/// `extend` (apply local transformation everywhere).
///
/// # Key Change from Legacy
/// The constraint is declared by the implementor via `BoundedHKT::Constraint`,
/// not hardcoded as `Zero + Copy + Clone`.
///
/// # Default Implementations
///
/// - `duplicate`: Default in terms of `extend`
/// - `fmap`: Default in terms of `extend` and `extract`
pub trait BoundedComonad<F: BoundedHKT>: BoundedFunctor<F> {
    /// Extract the value at the current focus.
    fn extract<A>(fa: &F::Type<A>) -> A
    where
        A: Satisfies<F::Constraint> + Clone;

    /// Extend a local observation to the entire structure.
    fn extend<A, B, Func>(fa: &F::Type<A>, f: Func) -> F::Type<B>
    where
        A: Satisfies<F::Constraint> + Clone,
        B: Satisfies<F::Constraint>,
        Func: FnMut(&F::Type<A>) -> B;

    /// Duplicate the structure (dual of join).
    ///
    /// Default implementation: `extend(fa, |x| x.clone())`
    fn duplicate<A>(fa: &F::Type<A>) -> F::Type<F::Type<A>>
    where
        A: Satisfies<F::Constraint> + Clone,
        F::Type<A>: Satisfies<F::Constraint> + Clone,
    {
        Self::extend(fa, |x| (*x).clone())
    }
}
```

#### 3.1.7 BoundedAdjunction Trait (Replacement)

```rust
// Location: deep_causality_haft/src/algebra/adjunction.rs (REPLACES existing)

/// Adjunction between constrained functors with runtime context.
///
/// # Key Change from Legacy
/// Constraint is declared by the implementor via `BoundedHKT::Constraint`,
/// not hardcoded as `Zero + Copy + PartialEq + Add + Mul`.
///
/// # Default Implementations
///
/// - `left_adjunct`: Default in terms of `unit` and `fmap`
/// - `right_adjunct`: Default in terms of `fmap` and `counit`
pub trait BoundedAdjunction<L: BoundedHKT, R: BoundedHKT, Context> {
    /// Left adjunct: (L<A> → B) → (A → R<B>)
    ///
    /// Default: `fmap(unit(a), f)`
    fn left_adjunct<A, B, F>(ctx: &Context, a: A, f: F) -> R::Type<B>
    where
        A: Satisfies<L::Constraint> + Satisfies<R::Constraint> + Clone,
        B: Satisfies<R::Constraint>,
        F: Fn(L::Type<A>) -> B;

    /// Right adjunct: (A → R<B>) → (L<A> → B)
    ///
    /// Default: `counit(fmap(la, f))`
    fn right_adjunct<A, B, F>(ctx: &Context, la: L::Type<A>, f: F) -> B
    where
        A: Satisfies<L::Constraint> + Clone,
        B: Satisfies<L::Constraint> + Satisfies<R::Constraint>,
        F: FnMut(A) -> R::Type<B>;

    /// Unit: A → R<L<A>>
    fn unit<A>(ctx: &Context, a: A) -> R::Type<L::Type<A>>
    where
        A: Satisfies<L::Constraint> + Satisfies<R::Constraint> + Clone;

    /// Counit: L<R<B>> → B
    fn counit<B>(ctx: &Context, lrb: L::Type<R::Type<B>>) -> B
    where
        B: Satisfies<L::Constraint> + Satisfies<R::Constraint> + Clone;
}
```

### 3.2 Constraint Definitions

#### 3.2.1 Standard Constraints Module

```rust
// Location: deep_causality_haft/src/core/constraints.rs

use crate::Satisfies;

/// Marker for types requiring no constraint (fully polymorphic).
/// Blanket impl: `impl<T> Satisfies<NoConstraint> for T {}`
pub struct NoConstraint;
impl<T> Satisfies<NoConstraint> for T {}

/// Marker for numeric types with Zero.
pub struct NumericConstraint;
impl<T: deep_causality_num::Zero + Copy + Clone> Satisfies<NumericConstraint> for T {}

/// Marker for types safe to send across threads.
pub struct ThreadSafeConstraint;
impl<T: Send + Sync + 'static> Satisfies<ThreadSafeConstraint> for T {}

/// Marker for clonable types.
pub struct CloneConstraint;
impl<T: Clone> Satisfies<CloneConstraint> for T {}
```

#### 3.2.2 TensorData Constraint (in deep_causality_tensor)

```rust
// Location: deep_causality_tensor/src/types/constraint.rs

use deep_causality_haft::Satisfies;
use crate::TensorData;

/// Constraint marker for tensor-compatible data types.
pub struct TensorDataConstraint;

/// All TensorData types satisfy this constraint.
impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}
```

### 3.3 Default Implementation Helpers

Default implementations are provided where sensible to reduce boilerplate. Since Rust traits
cannot provide default method implementations that depend on other traits not in the supertrait
chain, these are documented patterns rather than standalone functions.

**Pattern: `fmap` via iteration**

For collection-like types, `fmap` can be implemented by iterating and reconstructing:

```rust
// Example implementation pattern for Vec-like types
impl BoundedFunctor<VecWitness> for VecWitness {
    fn fmap<A, B, Func>(fa: Vec<A>, mut f: Func) -> Vec<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        fa.into_iter().map(f).collect()
    }
}
```

**Pattern: `apply` via `bind`**

When implementing both `BoundedApplicative` and `BoundedMonad`, `apply` can delegate to `bind`:

```rust
// In your BoundedApplicative impl, if you also implement BoundedMonad:
fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
where
    A: Satisfies<F::Constraint> + Clone,
    B: Satisfies<F::Constraint>,
    Func: FnMut(A) -> B + Clone,
{
    // Pseudocode: bind over f_ab, for each func, fmap over f_a
    // Actual implementation depends on the specific type
    Self::bind(f_ab, |func| Self::fmap(f_a.clone(), func))
}
```

**Pattern: `join` via `bind`**

The `join` method in `BoundedMonad` has a default implementation:

```rust
// Already shown in BoundedMonad trait definition:
fn join<A>(m_m_a: F::Type<F::Type<A>>) -> F::Type<A>
where
    A: Satisfies<F::Constraint>,
    F::Type<A>: Satisfies<F::Constraint>,
{
    Self::bind(m_m_a, |x| x)
}
```

---

## 4. Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Objective:** Replace core infrastructure.

#### 4.1.1 Files to Create

| File                                          | Purpose                              |
|-----------------------------------------------|--------------------------------------|
| `deep_causality_haft/src/core/constraint.rs`  | `Satisfies<C>` trait, `NoConstraint` |
| `deep_causality_haft/src/core/hkt_bounded.rs` | `BoundedHKT` trait                   |
| `deep_causality_haft/src/core/constraints.rs` | Standard constraint markers          |
| `deep_causality_haft/src/algebra/defaults.rs` | Default implementation helpers       |

#### 4.1.2 Files to Replace (Complete Rewrite)

| File                                            | Change                                       |
|-------------------------------------------------|----------------------------------------------|
| `deep_causality_haft/src/algebra/comonad.rs`    | Replace `BoundedComonad` with new version    |
| `deep_causality_haft/src/algebra/adjunction.rs` | Replace `BoundedAdjunction` with new version |

#### 4.1.3 Files to Modify

| File                                             | Change                                           |
|--------------------------------------------------|--------------------------------------------------|
| `deep_causality_haft/src/algebra/functor.rs`     | Add `BoundedFunctor` alongside `Functor`         |
| `deep_causality_haft/src/algebra/applicative.rs` | Add `BoundedApplicative` alongside `Applicative` |
| `deep_causality_haft/src/algebra/monad.rs`       | Add `BoundedMonad` alongside `Monad`             |

#### 4.1.4 lib.rs Updates

```rust
// New exports
pub use crate::core::constraint::{Satisfies, NoConstraint};
pub use crate::core::hkt_bounded::BoundedHKT;
pub use crate::core::constraints::{NumericConstraint, ThreadSafeConstraint, CloneConstraint};
pub use crate::algebra::functor_bounded::BoundedFunctor;
pub use crate::algebra::applicative_bounded::BoundedApplicative;
pub use crate::algebra::monad_bounded::BoundedMonad;
// BoundedComonad and BoundedAdjunction retain original names (replaced in-place)
```

### Phase 2: Tensor Crate Migration (Week 3-4)

**Objective:** Migrate `deep_causality_tensor` to new system.

#### 4.2.1 Files to Create

| File                                            | Purpose                           |
|-------------------------------------------------|-----------------------------------|
| `deep_causality_tensor/src/types/constraint.rs` | `TensorDataConstraint` definition |

#### 4.2.2 Files to Modify

| File                                              | Change                                          |
|---------------------------------------------------|-------------------------------------------------|
| `deep_causality_tensor/src/extensions/ext_hkt.rs` | Replace `BoundedComonad` impl, add `BoundedHKT` |
| `deep_causality_tensor/src/lib.rs`                | Export `TensorDataConstraint`                   |

### Phase 3: Multivector Crate Migration (Week 5)

**Objective:** Migrate `deep_causality_multivector` to new system.

| File                                                               | Change                    |
|--------------------------------------------------------------------|---------------------------|
| `deep_causality_multivector/src/extensions/hkt_multivector/mod.rs` | Replace all bounded impls |

### Phase 4: Topology Crate Migration (Week 6-7)

**Objective:** Migrate all topology witnesses.

| File                                                               | Change                      |
|--------------------------------------------------------------------|-----------------------------|
| `deep_causality_topology/src/extensions/hkt_manifold.rs`           | Replace `BoundedComonad`    |
| `deep_causality_topology/src/extensions/hkt_topology.rs`           | Replace `BoundedComonad`    |
| `deep_causality_topology/src/extensions/hkt_graph.rs`              | Replace `BoundedComonad`    |
| `deep_causality_topology/src/extensions/hkt_hypergraph.rs`         | Replace `BoundedComonad`    |
| `deep_causality_topology/src/extensions/hkt_point_cloud.rs`        | Replace `BoundedComonad`    |
| `deep_causality_topology/src/extensions/hkt_simplicial_complex.rs` | Replace `BoundedAdjunction` |

### Phase 5: Sparse Crate Migration (Week 8)

**Objective:** Migrate sparse matrix witness.

| File                                              | Change                                        |
|---------------------------------------------------|-----------------------------------------------|
| `deep_causality_sparse/src/extensions/ext_hkt.rs` | Replace `BoundedComonad`, `BoundedAdjunction` |

### Phase 6: Cleanup and Verification (Week 9-10)

**Objective:** Remove all legacy code, full test suite.

- Delete any temporary compatibility shims
- Full `make build && make test` verification
- Update all documentation
- Run benchmarks to verify no performance regression

---

## 5. Migration Plan

### 5.1 Types Requiring Migration

| Type                       | Crate                        | Legacy Traits to Replace              | Constraint             |
|----------------------------|------------------------------|---------------------------------------|------------------------|
| `CausalTensorWitness`      | `deep_causality_tensor`      | `BoundedComonad`, `BoundedAdjunction` | `TensorDataConstraint` |
| `CausalMultiVectorWitness` | `deep_causality_multivector` | `BoundedComonad`, `BoundedAdjunction` | `NumericConstraint`    |
| `ManifoldWitness`          | `deep_causality_topology`    | `BoundedComonad`                      | `NumericConstraint`    |
| `TopologyWitness`          | `deep_causality_topology`    | `BoundedComonad`                      | `NumericConstraint`    |
| `GraphWitness`             | `deep_causality_topology`    | `BoundedComonad`                      | `NumericConstraint`    |
| `HypergraphWitness`        | `deep_causality_topology`    | `BoundedComonad`                      | `NumericConstraint`    |
| `PointCloudWitness`        | `deep_causality_topology`    | `BoundedComonad`                      | `NumericConstraint`    |
| `CsrMatrixWitness`         | `deep_causality_sparse`      | `BoundedComonad`, `BoundedAdjunction` | `NumericConstraint`    |
| `ChainWitness`             | `deep_causality_topology`    | Used in `BoundedAdjunction`           | `NumericConstraint`    |

### 5.2 Migration Steps Per Type

#### Step 1: Define Constraint (if new)

```rust
// For types needing TensorData, use existing TensorDataConstraint
// For types needing Zero + Copy, use NumericConstraint (already defined)
```

#### Step 2: Implement `BoundedHKT`

```rust
impl BoundedHKT for ManifoldWitness {
    type Constraint = NumericConstraint;
    type Type<T> = Manifold<T>
    where
        T: Satisfies<NumericConstraint>;
}
```

#### Step 3: Replace Bounded Algebra Impls

```rust
// BEFORE (legacy - DELETE)
impl BoundedComonad<ManifoldWitness> for ManifoldWitness {
    fn extend<A, B, Func>(fa: &Manifold<A>, f: Func) -> Manifold<B>
    where
        A: Zero + Copy + Clone,  // HARDCODED
        B: Zero + Copy + Clone,  // HARDCODED
    { ... }
}

// AFTER (new - REPLACE)
impl BoundedComonad<ManifoldWitness> for ManifoldWitness {
    fn extend<A, B, Func>(fa: &Manifold<A>, f: Func) -> Manifold<B>
    where
        A: Satisfies<NumericConstraint> + Clone,  // FROM BoundedHKT
        B: Satisfies<NumericConstraint>,          // FROM BoundedHKT
    { ... }
}
```

### 5.3 Full Migration Example: CausalTensorWitness

**Legacy Implementation (to be deleted):**

```rust
// deep_causality_tensor/src/extensions/ext_hkt.rs:112-143
impl BoundedComonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Clone,
    { ... }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(&CausalTensor<A>) -> B,
        A: Zero + Copy + Clone,  // HARDCODED - TO BE REMOVED
        B: Zero + Copy + Clone,  // HARDCODED - TO BE REMOVED
    { ... }
}
```

**New Implementation (complete replacement):**

```rust
// deep_causality_tensor/src/extensions/ext_hkt.rs

use deep_causality_haft::{BoundedHKT, BoundedComonad, BoundedAdjunction, Satisfies};
use crate::{CausalTensor, TensorDataConstraint};

impl BoundedHKT for CausalTensorWitness {
    type Constraint = TensorDataConstraint;
    type Type<T> = CausalTensor<T>
    where
        T: Satisfies<TensorDataConstraint>;
}

impl BoundedComonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Satisfies<TensorDataConstraint> + Clone,
    {
        // Existing logic unchanged
        if fa.ndim() == 0 && !fa.is_empty() {
            fa.to_vec().into_iter().next().unwrap()
        } else if fa.is_empty() {
            panic!("CoMonad::extract cannot be called on an empty CausalTensor.");
        } else {
            fa.to_vec().into_iter().next().unwrap()
        }
    }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        A: Satisfies<TensorDataConstraint> + Clone,
        B: Satisfies<TensorDataConstraint>,
        Func: FnMut(&CausalTensor<A>) -> B,
    {
        // Existing logic unchanged - bounds now come from constraint
        let len = fa.len();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let focused_view = fa.shifted_view(i);
                f(&focused_view)
            })
            .collect();

        CausalTensor::from_slice(&new_data, fa.shape())
    }
}
```

---

## 6. Quality Assurance

### 6.1 Testing Requirements

| Category           | Requirement                            | Coverage Target |
|--------------------|----------------------------------------|-----------------|
| Unit Tests         | Each trait method tested independently | 100%            |
| Integration Tests  | Cross-crate usage patterns             | 90%             |
| Property Tests     | Functor/Monad law verification         | All laws        |
| Compile-Time Tests | Constraint violation detection         | N/A             |
| Benchmark Tests    | Performance regression prevention      | ±5% tolerance   |

### 6.2 Documentation Requirements

- [ ] Rustdoc for all public items
- [ ] Module-level documentation with examples
- [ ] Update `docs/HAFT.md` with new patterns
- [ ] Architecture decision record (ADR)

### 6.3 Code Review Checklist

- [ ] No `unsafe` code
- [ ] No dynamic dispatch (`dyn`)
- [ ] Static dispatch enforced
- [ ] Zero-cost abstraction principle maintained
- [ ] All bounds propagated correctly
- [ ] No legacy trait implementations remain

### 6.4 Production Readiness Criteria

| Criterion          | Requirement                                             |
|--------------------|---------------------------------------------------------|
| **Stability**      | All tests pass on stable Rust (MSRV 1.75+)              |
| **Performance**    | No measurable regression in benchmarks                  |
| **Compile Time**   | < 10% increase in crate compile time                    |
| **Documentation**  | 100% public API documented                              |
| **Legacy Removal** | Zero traces of old `BoundedComonad`/`BoundedAdjunction` |

---

## 7. Security & Safety Considerations

### 7.1 Safety Guarantees

- **No runtime checks avoided**: All constraint violations are compile-time errors
- **No unsafe code**: Entire implementation is safe Rust
- **Thread safety**: Constraint system can encode `Send + Sync` requirements

### 7.2 Zero Backward Compatibility

> [!CAUTION]
> **This is a breaking change by design.**

- All external code using `BoundedComonad` or `BoundedAdjunction` will break
- This is acceptable: zero external adoption exists
- Forward-evolving codebase prioritizes clean design over compatibility

---

## 8. Appendix: Full Type Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                      Constraint System                          │
├─────────────────────────────────────────────────────────────────┤
│  Satisfies<C> ◄──── NoConstraint       (impl<T> for T)          │
│                ◄──── NumericConstraint  (Zero + Copy + Clone)   │
│                ◄──── TensorDataConstraint (Field + Copy + ...)  │
│                ◄──── ThreadSafeConstraint (Send + Sync)         │
│                ◄──── CloneConstraint    (Clone)                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      BoundedHKT                                  │
│  type Constraint; type Type<T> where T: Satisfies<Constraint>   │
└─────────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
    BoundedFunctor    BoundedApplicative   BoundedComonad
            │                 │              (with defaults)
            │                 │                 │
            └────────┬────────┘                 │
                     ▼                          │
              BoundedMonad ◄────────────────────┘
              (with defaults)
                     │
                     ▼
             BoundedAdjunction
             (with defaults)
```

---

## 9. Design Analysis: Advanced Constraint Features

This section analyzes two advanced features that could extend the constraint system.

---

### 9.1 Constraint Composition: `Satisfies<A> + Satisfies<B>`

**Question:** Should a type be able to declare that it requires *multiple* constraints simultaneously?

#### 9.1.1 What It Means

Constraint composition would allow HKT witnesses to require that inner types satisfy multiple independent constraints.

The solution is to create a **composite constraint marker**:

```rust
// THIS WORKS in stable Rust
pub struct TensorDataThreadSafe;  // New composite marker

// Single blanket impl with explicit combined bounds
impl<T> Satisfies<TensorDataThreadSafe> for T
where
    T: TensorData + Send + Sync  // Explicit list of all requirements
{}

// Now use it in BoundedHKT
impl BoundedHKT for ParallelTensorWitness {
    type Constraint = TensorDataThreadSafe;  // Single marker
    type Type<T> = ParallelTensor<T>
    where
        T: Satisfies<TensorDataThreadSafe>;
}
```

This pattern is explicit, compiles on stable Rust, and avoids coherence issues.

#### 9.1.2 Current Design Impact

**The current design already supports this via manual composition:**

```rust
// Define a composite constraint
pub struct NumericThreadSafeConstraint;

// Blanket impl requires BOTH properties
impl<T> Satisfies<NumericThreadSafeConstraint> for T
where
    T: Zero + Copy + Clone + Send + Sync
{}
```

**No changes to `BoundedHKT` are needed** — the pattern works today.

#### 9.1.3 Practical Gains

| Gain                    | Description                                               |
|-------------------------|-----------------------------------------------------------|
| **Parallel physics**    | GPU tensor types often require `Send + Sync + TensorData` |
| **Safe sharing**        | Manifolds shared across threads need `Clone + Sync`       |
| **Layered constraints** | "This type is numeric AND can be serialized"              |

#### 9.1.4 Tradeoffs

| Tradeoff                      | Impact                                                                      |
|-------------------------------|-----------------------------------------------------------------------------|
| **Boilerplate**               | Each composite requires a new marker struct and blanket impl                |
| **No automatic intersection** | Can't write `Satisfies<A> + Satisfies<B>` in where clauses directly         |
| **Orphan rules**              | Composite constraints must be defined in crates that own at least one trait |

#### 9.1.5 Handling Orphan Rules

Rust's **orphan rules** (coherence rules) are critical when defining composite constraints.
Understanding them determines **where** you can legally define your `Satisfies` blanket impl.

**The Orphan Rule:**

> You can only implement a trait for a type if **at least one of** the following is true:
> 1. Your crate defines the **trait** being implemented
> 2. Your crate defines the **type** being implemented for
> 3. There is a "local type" in the impl that acts as a "covering" type

For blanket impls like `impl<T: SomeBounds> Satisfies<Marker> for T {}`, the key elements are:
- **Trait:** `Satisfies<Marker>` (from `deep_causality_haft`)
- **Type:** `T` (a generic type parameter — NOT a local type)
- **Marker:** The constraint marker struct

**Rule Application to Composite Constraints:**

Since `T` is generic and not a local type, you need **either**:
1. Your crate defines `Satisfies` (only true for `deep_causality_haft`)
2. Your crate defines the `Marker` struct

**This means: the crate that defines the `Marker` struct must also define the blanket impl.**

**Example — Valid Placements:**

```rust
// ✅ VALID: deep_causality_haft defines both Satisfies and NoConstraint
// Location: deep_causality_haft/src/core/constraint.rs
pub struct NoConstraint;
impl<T> Satisfies<NoConstraint> for T {}

// ✅ VALID: deep_causality_tensor defines TensorDataConstraint
// Location: deep_causality_tensor/src/types/constraint.rs
pub struct TensorDataConstraint;
impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}

// ✅ VALID: deep_causality_haft defines NumericConstraint
// Location: deep_causality_haft/src/core/constraints.rs
pub struct NumericConstraint;
impl<T: Zero + Copy + Clone> Satisfies<NumericConstraint> for T {}
```

**Example — Invalid Placement (Orphan Violation):**

```rust
// ❌ INVALID: deep_causality_topology cannot define this
// Because it owns neither Satisfies nor TensorDataConstraint
// Location: deep_causality_topology/src/??? — WILL NOT COMPILE

// This would be an orphan impl:
impl<T: TensorData + Send + Sync> Satisfies<TensorDataConstraint> for T {}
// Error: cannot implement trait `Satisfies` for generic type `T`
// because neither the trait nor the type is defined in this crate
```

**Decision Tree for Constraint Placement:**

```
Where should I define my composite constraint?

┌──────────────────────────────────────────────────────────────────┐
│ Q1: Does the constraint use ONLY traits from deep_causality_haft │
│     or Rust std (Zero, Copy, Clone, Send, Sync, etc.)?           │
├──────────────────────────────────────────────────────────────────┤
│ YES → Define in deep_causality_haft/src/core/constraints.rs      │
│       Examples: NumericConstraint, ThreadSafeConstraint          │
├──────────────────────────────────────────────────────────────────┤
│ NO → Q2: Does the constraint use TensorData?                     │
│     ├─ YES → Define in deep_causality_tensor                     │
│     │        Examples: TensorDataConstraint                      │
│     └─ NO → Q3: Does the constraint use a domain-specific trait? │
│          └─ Define in the crate that owns that trait             │
└──────────────────────────────────────────────────────────────────┘
```

**Composite Constraint Placement Rules:**

| Constraint Type | Required Traits | Define In |
|-----------------|-----------------|-----------|
| `NumericConstraint` | `Zero + Copy + Clone` | `deep_causality_haft` |
| `ThreadSafeConstraint` | `Send + Sync` | `deep_causality_haft` |
| `TensorDataConstraint` | `TensorData` | `deep_causality_tensor` |
| `NumericThreadSafe` | `Zero + Copy + Send + Sync` | `deep_causality_haft` |
| `TensorDataThreadSafe` | `TensorData + Send + Sync` | `deep_causality_tensor` |
| `ManifoldConstraint` | `ManifoldData` (hypothetical) | `deep_causality_topology` |

**Key Insight:**

The crate hierarchy for constraints follows the dependency graph:

```
deep_causality_haft          ← Defines Satisfies, NoConstraint, NumericConstraint
        ↓
deep_causality_num           ← Defines Zero, One, Field (used in constraints)
        ↓
deep_causality_tensor        ← Defines TensorData, TensorDataConstraint
        ↓
deep_causality_topology      ← Can ONLY define constraints using its OWN traits
deep_causality_multivector   ← Can ONLY define constraints using its OWN traits
```

**Downstream crates cannot re-export or extend upstream constraints** — they can only:
1. Use existing constraints from upstream crates
2. Define NEW constraint markers for traits they own

#### 9.1.6 Recommendation

> [!TIP]
> **RECOMMENDATION: No additional support needed.**
>
> The current design handles composition via manual composite markers.
> This is explicit, readable, and avoids complex type-level machinery.
>
> **Action:** Document the pattern in `docs/HAFT.md` with examples.

---

### 9.2 Higher-Order Constraints: Constraints Referencing Constraints

**Question:** Should constraints be able to reference or extend other constraints?

#### 9.2.1 What It Means

Higher-order constraints would allow building constraint hierarchies. This is
**NOT recommended and NOT implemented** due to coherence issues in stable Rust.

```rust
// CONCEPTUAL ONLY - This pattern has coherence issues in practice
// Shown here to explain WHY we don't support it

pub trait ConstraintExtends<Base: ?Sized> {}

// TensorDataConstraint "extends" NumericConstraint
impl ConstraintExtends<NumericConstraint> for TensorDataConstraint {}

// The following blanket impl is where problems arise:
// It would allow automatic Satisfies<NumericConstraint> for any T: Satisfies<TensorDataConstraint>
impl<T, C, B> Satisfies<B> for T
where
    T: Satisfies<C>,
    C: ConstraintExtends<B>,
{}
// PROBLEM: This conflicts with other Satisfies impls and creates coherence errors
```

This would create a **constraint subtyping lattice**:

```
         NoConstraint (⊤ - everything)
              │
     ┌────────┼────────┐
     ▼        ▼        ▼
  Clone   ThreadSafe  Numeric
     │        │        │
     └────────┼────────┘
              ▼
         TensorData (⊥ - most restrictive)
```

#### 9.2.2 Current Design Impact

**Rust does not support this natively.** Implementing it would require:

1. A `ConstraintExtends<Base>` trait
2. Complex blanket impls with potential coherence issues
3. Negative bounds or specialization (unstable features)

**The current design explicitly avoids this** — each constraint is independent.

#### 9.2.3 Practical Gains

| Gain                           | Description                                                                    |
|--------------------------------|--------------------------------------------------------------------------------|
| **Automatic coercion**         | A `Tensor<T: TensorData>` could be used where `Tensor<T: Numeric>` is expected |
| **DRY constraint definitions** | Define `TensorData = Numeric + Field + PartialOrd` once                        |
| **Variance-like behavior**     | Constraints form a natural lattice for subtyping                               |

#### 9.2.4 Tradeoffs

| Tradeoff                   | Impact                                                  | Severity        |
|----------------------------|---------------------------------------------------------|-----------------|
| **Coherence violations**   | Overlapping impls when multiple inheritance paths exist | 🔴 **Critical** |
| **Compile-time explosion** | Trait solver must explore inheritance graph             | 🟡 Moderate     |
| **Complexity**             | Debugging constraint errors becomes much harder         | 🟡 Moderate     |
| **Unstable features**      | Likely requires `specialization` or `negative_impls`    | 🔴 **Critical** |

#### 9.2.5 Example of Coherence Problem

```rust
// CONCEPTUAL - Demonstrates why higher-order constraints fail
// These impls would create overlapping implementations:

impl ConstraintExtends<NoConstraint> for NumericConstraint {}
impl ConstraintExtends<NoConstraint> for CloneConstraint {}
impl ConstraintExtends<NumericConstraint> for TensorDataConstraint {}
impl ConstraintExtends<CloneConstraint> for TensorDataConstraint {}

// With the blanket impl from 9.2.1, the compiler would try to derive:
//   impl<T: Satisfies<TensorDataConstraint>> Satisfies<NoConstraint> for T
// via BOTH:
//   - TensorDataConstraint -> NumericConstraint -> NoConstraint
//   - TensorDataConstraint -> CloneConstraint -> NoConstraint
// This creates conflicting impl paths that Rust's coherence checker rejects.
```

#### 9.2.6 Recommendation

> [!CAUTION]
> **RECOMMENDATION: Do NOT implement higher-order constraints.**
>
> The complexity-to-benefit ratio is unfavorable:
> - Rust stable cannot express this safely
> - Coherence issues are fundamental, not implementation bugs
> - Manual constraint composition (§9.1) covers 95% of use cases
>
> **Action:** Explicitly document that constraints are **flat** (no hierarchy).
> If a type needs multiple properties, define a composite constraint manually.

---

### 9.3 Summary of Decisions

| Feature                              | Recommendation                    | Rationale                                |
|--------------------------------------|-----------------------------------|------------------------------------------|
| **Constraint Composition** (`A + B`) | ✅ Supported via manual composites | Already works; document the pattern      |
| **Higher-Order Constraints**         | ❌ Not supported                   | Coherence issues; requires unstable Rust |

---

## 10. References

1. `AGENTS.md` - Project conventions and structure
2. `specs/current/hkt_fields.md` - TensorData incompatibility analysis
3. `specs/current/topo_physics.md` - Physics use cases for HKT
4. `deep_causality_haft/src/algebra/comonad.rs` - Legacy `BoundedComonad` (to be replaced)
5. `deep_causality_haft/src/algebra/adjunction.rs` - Legacy `BoundedAdjunction` (to be replaced)
6. Rust RFC 1598: Generic Associated Types (GAT)
7. Haskell `ConstraintKinds` for constraint polymorphism
8. Rust `specialization` RFC (unstable) - Why higher-order constraints are problematic
