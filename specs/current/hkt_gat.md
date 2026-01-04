# Unified GAT-Bounded Higher-Kinded Types for Rust: Product Specification

* **Product Area:** Deep Causality
* **Crate:** `deep_causality_haft`  
* **Status:** Defered
* **Target:** Jan 2026   
* **Classification:** Core Infrastructure  
* **Owner:** DeepCausality Authors


Note:

> **Strict GAT HKTs are Solved in the Next-Generation Trait Solver**

As of **January 2026**, we have confirmed that the inability to implement strict `Monad` and `CoMonad` (due to `E0276`/ `E0277` GAT normalization errors) is a **temporary limitation** of the current stable Rust trait solver.

**Verification:**
Using the nightly compiler with the new trait solver flag (`-Znext-solver`), the strict implementations for `StrictCausalTensorWitness` **compile successfully without modification**.

**Implication for DeepCausality:**
1. The **Dual-Witness Pattern** (unbounded vs strict) is a transitional architecture.
2. Once the new trait solver stabilizes, we can unify the design into a single, fully constrained HKT witness where `type Constraint` is universally enforced.
3. The current codebase contains commented-out strict implementations that are "future-proof" and ready to be enabled instantly when the compiler tooling matures.

For details, see hkt_gat_review.md document

---

## 1. Executive Summary

This document specifies a **Unified GAT-Bounded HKT** system for the `deep_causality_haft` crate. The design
**eliminates the distinction between bounded and unbounded HKTs** by treating unconstrained types as the special
case `Constraint = NoConstraint`. This results in a single, clean trait hierarchy that supports all types —
from simple `Vec<T>` to complex physics types requiring `TensorData` bounds.

### 1.1 Problem Statement

The current architecture has **two parallel trait hierarchies**:

| Category  | Traits                                | Limitation                             |
|-----------|---------------------------------------|----------------------------------------|
| Unbounded | `HKT`, `Functor`, `Monad`, `CoMonad`  | Cannot work with `T: TensorData` types |
| Bounded   | `BoundedComonad`, `BoundedAdjunction` | Hardcoded `Zero + Copy` bounds         |

This creates API fragmentation, code duplication, and forces users to choose between hierarchies.

### 1.2 Solution Overview

**Unify into a single hierarchy** where the constraint is an associated type:

```rust
pub trait Satisfies<C: ?Sized> {}
pub struct NoConstraint;
impl<T> Satisfies<NoConstraint> for T {}  // Everything satisfies NoConstraint

pub trait HKT {
    type Constraint: ?Sized;
    type Type<T>
    where
        T: Satisfies<Self::Constraint>;
}
```

- **Unconstrained types** use `type Constraint = NoConstraint;`
- **Constrained types** use `type Constraint = TensorDataConstraint;` (etc.)
- **Same traits for both** — no `Bounded` prefix needed

### 1.3 Key Decisions

> [!IMPORTANT]
> **Migration Strategy: Complete Replacement**

| Decision                      | Rationale                                             |
|-------------------------------|-------------------------------------------------------|
| **Unified hierarchy**         | Single set of traits for all types                    |
| **No backward compatibility** | Clean break justified by zero external adoption       |
| **No feature flags**          | Simple rip-and-replace                                |
| **Legacy traits removed**     | `HKT`, `Functor`, etc. replaced with unified versions |
| **Default implementations**   | Leveraged throughout to reduce boilerplate            |

---

## 2. Background & Motivation

### 2.1 The Unification Insight

Since `impl<T> Satisfies<NoConstraint> for T {}`, these are equivalent:

```rust
// OLD unbounded:
type Type<T>;

// NEW unified with NoConstraint:
type Type<T>
where
    T: Satisfies<NoConstraint>;  // Always satisfied!
```

**Result**: Unbounded HKT is just a special case of bounded HKT.

### 2.2 Types Affected

| Witness Type               | Current Constraint        | Unified Constraint          |
|----------------------------|---------------------------|-----------------------------|
| `VecWitness`               | (unbounded)               | `NoConstraint`              |
| `OptionWitness`            | (unbounded)               | `NoConstraint`              |
| `BoxWitness`               | (unbounded)               | `NoConstraint`              |
| `CausalTensorWitness`      | `Zero + Copy` (hardcoded) | `TensorDataConstraint`      |
| `CausalMultiVectorWitness` | `Zero + Copy` (hardcoded) | `AssociativeRingConstraint` |
| `ManifoldWitness`          | `Zero + Copy` (hardcoded) | `FieldConstraint`           |
| `CausalMultiFieldWitness`  | (new)                     | `TensorDataConstraint`      |
| `TopologyWitness`          | `Zero + Copy` (hardcoded) | `RealFieldConstraint`       |
| `GraphWitness`             | `Zero + Copy` (hardcoded) | `RealFieldConstraint`       |
| `CsrMatrixWitness`         | `Zero + Copy` (hardcoded) | `FieldConstraint`           |

---

## 3. Technical Specification

### 3.1 Core Traits

#### 3.1.1 Constraint System

```rust
// Location: deep_causality_haft/src/core/constraint.rs

/// Marker trait indicating that type `T` satisfies constraint `C`.
pub trait Satisfies<C: ?Sized> {}

/// The universal constraint — everything satisfies it.
pub struct NoConstraint;
impl<T> Satisfies<NoConstraint> for T {}

/// Thread-safe types.
pub struct ThreadSafeConstraint;
impl<T: Send + Sync> Satisfies<ThreadSafeConstraint> for T {}

/// Numeric types with zero element.
pub struct NumericConstraint;
impl<T: deep_causality_num::Zero + Copy + Clone> Satisfies<NumericConstraint> for T {}

/// Clonable types.
pub struct CloneConstraint;
impl<T: Clone> Satisfies<CloneConstraint> for T {}
```

#### 3.1.2 HKT Trait (Unified)

```rust
// Location: deep_causality_haft/src/core/hkt.rs
// REPLACES the old HKT trait

/// Unified Higher-Kinded Type with declarative constraint.
///
/// # For Unconstrained Types
/// ```rust
/// impl HKT for VecWitness {
///     type Constraint = NoConstraint;
///     type Type<T> = Vec<T> where T: Satisfies<NoConstraint>;
/// }
/// ```
///
/// # For Constrained Types
/// ```rust
/// impl HKT for CausalTensorWitness {
///     type Constraint = TensorDataConstraint;
///     type Type<T> = CausalTensor<T> where T: Satisfies<TensorDataConstraint>;
/// }
/// ```
pub trait HKT {
    /// The constraint on inner types. Use `NoConstraint` for fully polymorphic.
    type Constraint: ?Sized;

    /// The type constructor.
    type Type<T>
    where
        T: Satisfies<Self::Constraint>;
}
```

#### 3.1.3 Functor Trait (Unified)

```rust
// Location: deep_causality_haft/src/algebra/functor.rs
// REPLACES the old Functor trait

/// Functor for type constructors with any constraint.
///
/// # Laws
/// 1. Identity: `fmap(fa, |x| x) == fa`
/// 2. Composition: `fmap(fmap(fa, g), f) == fmap(fa, |x| f(g(x)))`
pub trait Functor<F: HKT> {
    fn fmap<A, B, Func>(fa: F::Type<A>, f: Func) -> F::Type<B>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> B;
}
```

#### 3.1.4 Applicative Trait (Unified)

```rust
// Location: deep_causality_haft/src/algebra/applicative.rs
// REPLACES the old Applicative trait

/// Applicative functor with constraint support.
pub trait Applicative<F: HKT>: Functor<F> {
    /// Lift a value into the context.
    fn pure<T>(value: T) -> F::Type<T>
    where
        T: Satisfies<F::Constraint>;

    /// Apply a wrapped function to a wrapped value.
    fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
    where
        A: Satisfies<F::Constraint> + Clone,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> B;
}
```

#### 3.1.5 Monad Trait (Unified)

```rust
// Location: deep_causality_haft/src/algebra/monad.rs
// REPLACES the old Monad trait

/// Monad with constraint support.
///
/// # Laws
/// 1. Left Identity: `bind(pure(a), f) == f(a)`
/// 2. Right Identity: `bind(m, pure) == m`
/// 3. Associativity: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`
pub trait Monad<F: HKT>: Applicative<F> {
    /// Bind (flatMap) operation.
    fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> F::Type<B>;

    /// Flatten nested structure. Default implementation provided.
    fn join<A>(m_m_a: F::Type<F::Type<A>>) -> F::Type<A>
    where
        A: Satisfies<F::Constraint>,
        F::Type<A>: Satisfies<F::Constraint>,
    {
        Self::bind(m_m_a, |x| x)
    }
}
```

#### 3.1.6 CoMonad Trait (Unified)

```rust
// Location: deep_causality_haft/src/algebra/comonad.rs
// REPLACES both old CoMonad and BoundedComonad

/// Comonad with constraint support.
///
/// The dual of Monad: extract (vs pure) and extend (vs bind).
pub trait CoMonad<F: HKT>: Functor<F> {
    /// Extract the focused value.
    fn extract<A>(fa: &F::Type<A>) -> A
    where
        A: Satisfies<F::Constraint> + Clone;

    /// Extend a local computation to the entire structure.
    fn extend<A, B, Func>(fa: &F::Type<A>, f: Func) -> F::Type<B>
    where
        A: Satisfies<F::Constraint> + Clone,
        B: Satisfies<F::Constraint>,
        Func: FnMut(&F::Type<A>) -> B;

    /// Duplicate the structure. Default implementation provided.
    fn duplicate<A>(fa: &F::Type<A>) -> F::Type<F::Type<A>>
    where
        A: Satisfies<F::Constraint> + Clone,
        F::Type<A>: Satisfies<F::Constraint> + Clone,
    {
        Self::extend(fa, |x| x.clone())
    }
}
```

#### 3.1.7 Adjunction Trait (Unified)

```rust
// Location: deep_causality_haft/src/algebra/adjunction.rs
// REPLACES both old Adjunction and BoundedAdjunction

/// Adjunction between two HKT functors with runtime context.
///
/// Models the duality: L ⊣ R (L is left adjoint to R).
pub trait Adjunction<L: HKT, R: HKT, Context> {
    /// Unit: A → R<L<A>>
    fn unit<A>(ctx: &Context, a: A) -> R::Type<L::Type<A>>
    where
        A: Satisfies<L::Constraint> + Satisfies<R::Constraint> + Clone;

    /// Counit: L<R<B>> → B
    fn counit<B>(ctx: &Context, lrb: L::Type<R::Type<B>>) -> B
    where
        B: Satisfies<L::Constraint> + Satisfies<R::Constraint> + Clone;

    /// Left adjunct: (L<A> → B) → (A → R<B>)
    fn left_adjunct<A, B, Func>(ctx: &Context, a: A, f: Func) -> R::Type<B>
    where
        A: Satisfies<L::Constraint> + Satisfies<R::Constraint> + Clone,
        B: Satisfies<R::Constraint>,
        Func: Fn(L::Type<A>) -> B;

    /// Right adjunct: (A → R<B>) → (L<A> → B)
    fn right_adjunct<A, B, Func>(ctx: &Context, la: L::Type<A>, f: Func) -> B
    where
        A: Satisfies<L::Constraint> + Clone,
        B: Satisfies<L::Constraint> + Satisfies<R::Constraint>,
        Func: FnMut(A) -> R::Type<B>;
}
```

### 3.2 Constraint Definitions

#### 3.2.1 Design Philosophy: Why Algebraic Constraints?

> [!IMPORTANT]
> **This is not typical in numerical libraries.**

Most numerical libraries treat bounds as implementation details:

- "Requires `T: Copy + Add`" — *What can I call on T?*
- NumPy, nalgebra, ndarray: Duck-typed or trait-soup bounds

DeepCausality takes a **mathematically principled** approach:

- "Requires `T: Field`" — *What algebraic structure does T form?*
- Constraints reflect **category theory** and **abstract algebra**

**Why this matters for physics:**

| Physics Domain            | Required Algebra         | Why                                                |
|---------------------------|--------------------------|----------------------------------------------------|
| Real-valued PDE           | `Field`                  | Division, commutativity for numerical methods      |
| Quaternion rotation       | `AssociativeRing`        | Non-commutative but associative multiplication     |
| Octonion electromagnetism | `AbelianGroup`           | Only additive structure (multiplication undefined) |
| Spinor calculus           | `Module<Complex>`        | Vector space over complex scalars                  |
| Dixon algebra             | `AssociativeRing` nested | Nested non-commutative algebras                    |

**The wisdom**: When a HKT declares `Constraint = FieldConstraint`, it's not just saying
"I need division" — it's declaring *this container participates in the category of Field-typed structures*.
This enables:

1. **Compile-time algebraic safety**: You cannot accidentally use Octonions (non-associative) where
   the algorithm requires associative multiplication.

2. **Self-documenting APIs**: `CausalMultiVectorWitness { Constraint = AssociativeRingConstraint }`
   immediately tells you Quaternions work but Octonions don't.

3. **Mathematical composability**: Adjunctions between `Field`-constrained and `Ring`-constrained
   HKTs can be typed precisely.

---

#### 3.2.2 The Algebraic Hierarchy

The constraints mirror `deep_causality_num`'s algebraic structure:

```
                    NoConstraint (all types)
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
      CloneConstraint  ThreadSafe  AbelianGroupConstraint
                                       │
                              RingConstraint
                                       │
                   ┌───────────────────┼───────────────────┐
                   ▼                   ▼                   ▼
      AssociativeRingConstraint  CommutativeRingConstraint  Distributive
                   │                   │
                   └─────────┬─────────┘
                             ▼
                      FieldConstraint
                             │
                      RealFieldConstraint
                             │
                      TensorDataConstraint (Field + Copy + Ord + Send + Sync)
```

**Each level adds structure:**

- `AbelianGroup`: Addition, subtraction, zero (linear combinations)
- `Ring`: Above + multiplication, one (polynomials, matrices)
- `AssociativeRing`: Above + `(ab)c = a(bc)` (Quaternions, Matrices)
- `CommutativeRing`: Ring + `ab = ba` (Integers, polynomials)
- `Field`: CommutativeRing + division (Reals, Complex, Rationals)
- `RealField`: Field + ordering + transcendentals (f32, f64)

---

#### 3.2.2a Concrete Type Algebra Implementation

| Type                     | Implements                                                      | Does NOT Implement                           | Use Constraint              |
|--------------------------|-----------------------------------------------------------------|----------------------------------------------|-----------------------------|
| `f32`, `f64`             | `RealField`, `Field`, `CommutativeRing`, `Ring`, `AbelianGroup` | —                                            | `RealFieldConstraint`       |
| `Complex<f64>`           | `Field`, `CommutativeRing`, `Ring`, `AbelianGroup`              | `RealField` (no ordering)                    | `FieldConstraint`           |
| `Quaternion<f64>`        | `AssociativeRing`, `Ring`, `AbelianGroup`                       | `CommutativeRing`, `Field` (non-commutative) | `AssociativeRingConstraint` |
| `Octonion<f64>`          | `AbelianGroup`                                                  | `Ring` (non-associative multiplication)      | `AbelianGroupConstraint`    |
| `Matrix<N, M, f64>`      | `AssociativeRing` (square), `Ring`, `AbelianGroup`              | `CommutativeRing` (AB ≠ BA)                  | `AssociativeRingConstraint` |
| `i8`–`i128`, `u8`–`u128` | `CommutativeRing`, `Ring`, `AbelianGroup`                       | `Field` (no division)                        | See note below              |
| `CausalTensor<f64>`      | `TensorData` (implies `RealField` + extras)                     | —                                            | `TensorDataConstraint`      |

---

#### 3.2.2b Known Limitation: Integer Types

> [!WARNING]
> **No unified integer constraint exists.**

Integer types (`i8`–`i128`, `u8`–`u128`) implement `CommutativeRing` but not `Field` (no division).
Currently, there is no `IntegerConstraint` or `CommutativeRingConstraint` specifically for integers.

**Why this is acceptable:**

1. **Physics equations rarely solve to integers**: Conservation laws, differential equations, field
   theories — virtually all require division and produce real or complex results.

2. **When integers appear, they're typically indices**: Loop counters, array indices, discrete counts
   — these don't participate in HKT composition.

3. **Workaround exists**: Use `RingConstraint` if you truly need integer-compatible algebra.
   This accepts integers but also accepts non-commutative types like Quaternions.

**Future consideration**: If integer arithmetic becomes a priority (e.g., discrete lattice QCD),
add `IntegerRingConstraint` with `impl<T: CommutativeRing + Not<Field>> Satisfies<...>` — but this
requires negative bounds (unstable Rust) or manual marker impls.

---

#### 3.2.2c Algebraic Isomorphism: Quaternions ≅ 2×2 Complex Matrices ≅ SU(2)

> [!TIP]
> **Types sharing the same algebraic constraint may be isomorphic — enabling GPU acceleration.**

Both `Quaternion<f64>` and `Matrix<2,2,Complex<f64>>` satisfy `AssociativeRingConstraint`.
More importantly, there exists a **faithful representation** (ring isomorphism):

```
Unit Quaternion: q = a + bi + cj + dk  (|q| = 1)

≅ SU(2) Matrix (2×2 unitary, det = 1):

   ┌                    ┐
   │  a + bi    c + di  │
   │ -c + di    a - bi  │
   └                    ┘

Quaternion multiplication = Matrix multiplication (exactly)
```

**Why this matters for quantum computing:**

| Concept                   | Quaternion View          | Matrix View         | GPU Acceleration  |
|---------------------------|--------------------------|---------------------|-------------------|
| **Single qubit gate**     | Unit quaternion rotation | SU(2) matrix        | ✅ Batched matmul  |
| **Pauli-X, Y, Z**         | `i`, `j`, `k` basis      | σ₁, σ₂, σ₃ matrices | ✅ Native support  |
| **Hadamard gate**         | `(1 + i)/√2`             | H = (σ₁ + σ₃)/√2    | ✅ Native support  |
| **Gate composition**      | Quaternion product       | Matrix product      | ✅ 10-100× speedup |
| **Bloch sphere rotation** | q · ψ · q⁻¹              | U ·                 | ψ⟩                | ✅ Native support |

**Practical use case: Bulk SU(2) gate simulation**

```rust
// Simulating 10,000 random single-qubit gates on 1,000 qubits

// CPU path: quaternion multiplication (slow)
let gates: Vec<Quaternion<f64> > = generate_random_su2_gates(10_000);
let states: Vec<Quaternion<f64> > = initial_bloch_states(1_000);
// O(10M) scalar operations, sequential

// MLX path: convert to batched 2×2 complex matrices
let gate_matrices: CausalTensor<Complex<f32> > = quats_to_su2_batch( & gates); // [10000, 2, 2]
let state_vectors: CausalTensor<Complex<f32> > = bloch_to_spinor( & states);   // [1000, 2, 1]
let result = gate_matrices.batched_matmul( & state_vectors); // GPU accelerated!
// Single MLX kernel, 10-100× faster
```

**HKT connection**: Because both representations satisfy `AssociativeRingConstraint`,
generic algorithms written over that constraint work for either representation:

```rust
fn compose_gates<T>(gates: impl Iterator<Item=T>) -> T
where
    T: Satisfies<AssociativeRingConstraint> + Copy + One,
{
    gates.fold(T::one(), |acc, g| acc * g)  // Works for Quaternion OR SU(2) matrix!
}
```

**Key insight**: The algebraic constraint system enables type-safe substitution between
isomorphic representations, allowing automatic GPU acceleration without changing algorithm logic.

#### 3.2.3 Standard Constraints in `deep_causality_haft`

```rust
// Location: deep_causality_haft/src/core/constraints.rs

use deep_causality_num::{
    AbelianGroup, Ring, AssociativeRing, CommutativeRing, Field, RealField, Module
};

// ============================================================================
// TIER 0: Universal Constraints
// ============================================================================

/// No constraint — all types satisfy this.
pub struct NoConstraint;
impl<T> Satisfies<NoConstraint> for T {}

/// Clonable types.
pub struct CloneConstraint;
impl<T: Clone> Satisfies<CloneConstraint> for T {}

/// Thread-safe types.
pub struct ThreadSafeConstraint;
impl<T: Send + Sync> Satisfies<ThreadSafeConstraint> for T {}

// ============================================================================
// TIER 1: Additive Algebra (Linear Combinations)
// Use Case: Octonion buffers, accumulators, superposition states
// ============================================================================

/// Abelian Group: Add + Sub + Zero, commutative addition.
/// Minimal requirement for linear combinations.
pub struct AbelianGroupConstraint;
impl<T: AbelianGroup + Copy> Satisfies<AbelianGroupConstraint> for T {}

// ============================================================================
// TIER 2: Multiplicative Algebra (Ring Operations)
// Use Case: Matrix algebra, polynomial evaluation
// ============================================================================

/// Ring: AbelianGroup + Mul + One + Distributive.
/// No commutativity or associativity guarantees on multiplication.
pub struct RingConstraint;
impl<T: Ring + Copy> Satisfies<RingConstraint> for T {}

/// Associative Ring: Ring where (ab)c = a(bc).
/// Allows Quaternions, Matrices, but NOT Octonions.
pub struct AssociativeRingConstraint;
impl<T: AssociativeRing + Copy> Satisfies<AssociativeRingConstraint> for T {}

/// Commutative Ring: Ring where ab = ba.
/// Allows Integers, Polynomials, but NOT Matrices.
pub struct CommutativeRingConstraint;
impl<T: CommutativeRing + Copy> Satisfies<CommutativeRingConstraint> for T {}

// ============================================================================
// TIER 3: Division Algebra (Field Operations)
// Use Case: Standard numerical computing, Clifford algebra
// ============================================================================

/// Field: CommutativeRing + Division.
/// Standard numerical type constraint for most algorithms.
pub struct FieldConstraint;
impl<T: Field + Copy> Satisfies<FieldConstraint> for T {}

/// Real Field: Field + Ordering + Transcendentals.
/// Required for algorithms using sqrt, sin, cos, comparisons.
pub struct RealFieldConstraint;
impl<T: RealField + Copy> Satisfies<RealFieldConstraint> for T {}

// ============================================================================
// TIER 4: Composite Constraints
// ============================================================================

/// Field + Thread Safety.
pub struct FieldThreadSafe;
impl<T: Field + Copy + Send + Sync> Satisfies<FieldThreadSafe> for T {}

/// Real Field + Thread Safety.
pub struct RealFieldThreadSafe;
impl<T: RealField + Copy + Send + Sync> Satisfies<RealFieldThreadSafe> for T {}
```

---

#### 3.2.4 Domain-Specific Constraints

```rust
// Location: deep_causality_tensor/src/types/constraint.rs

/// TensorData: The full physics stack.
/// Field + Copy + Default + PartialOrd + Send + Sync + 'static
pub struct TensorDataConstraint;
impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}

/// TensorData + additional thread guarantees.
pub struct TensorDataThreadSafe;
impl<T: TensorData + Send + Sync> Satisfies<TensorDataThreadSafe> for T {}
```

---

#### 3.2.5 Algebraic Constraint Selection Guide

| Your Type           | Algebraic Properties                 | Use Constraint              |
|---------------------|--------------------------------------|-----------------------------|
| `f32`, `f64`        | Real field                           | `RealFieldConstraint`       |
| `Complex<f64>`      | Field (commutative)                  | `FieldConstraint`           |
| `Quaternion<f64>`   | Associative ring (non-commutative)   | `AssociativeRingConstraint` |
| `Octonion<f64>`     | Abelian group only (non-associative) | `AbelianGroupConstraint`    |
| `Matrix<f64>`       | Associative ring (non-commutative)   | `AssociativeRingConstraint` |
| `CausalTensor<f64>` | Real field + ordering + thread-safe  | `TensorDataConstraint`      |
| `String`, `Vec<u8>` | No algebraic structure               | `NoConstraint`              |

---

#### 3.2.6 Example: Tier-Based Algorithm Dispatch

```rust
impl<T> CausalMultiVector<T> {
    // TIER 1: Only needs AbelianGroup — works for Octonions!
    pub fn add(&self, rhs: &Self) -> Self
    where
        T: Satisfies<AbelianGroupConstraint> + Copy,
    { ... }

    // TIER 2: Needs AssociativeRing — Quaternions work, Octonions BLOCKED
    pub fn geometric_product(&self, rhs: &Self) -> Self
    where
        T: Satisfies<AssociativeRingConstraint> + Copy,
    { ... }

    // TIER 3: Needs Field — only commutative types (Complex, Real)
    pub fn inverse(&self) -> Self
    where
        T: Satisfies<FieldConstraint> + Copy,
    { ... }
}
```

**Compile-time safety**: Calling `geometric_product` with `Octonion` fails at compile time
because `Octonion` does not implement `AssociativeRing`, so it doesn't satisfy
`AssociativeRingConstraint`.

---

## 4. Implementation Examples

### 4.1 Unconstrained Type: Vec

```rust
// deep_causality_haft/src/extensions/hkt_vec_ext.rs

pub struct VecWitness;

impl HKT for VecWitness {
    type Constraint = NoConstraint;
    type Type<T> = Vec<T>
    where
        T: Satisfies<NoConstraint>;
}

impl Functor<VecWitness> for VecWitness {
    fn fmap<A, B, Func>(fa: Vec<A>, mut f: Func) -> Vec<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        fa.into_iter().map(f).collect()
    }
}

impl Applicative<VecWitness> for VecWitness {
    fn pure<T>(value: T) -> Vec<T>
    where
        T: Satisfies<NoConstraint>,
    {
        vec![value]
    }

    fn apply<A, B, Func>(f_ab: Vec<Func>, f_a: Vec<A>) -> Vec<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        f_ab.into_iter()
            .flat_map(|mut func| f_a.clone().into_iter().map(move |a| func(a)))
            .collect()
    }
}

impl Monad<VecWitness> for VecWitness {
    fn bind<A, B, Func>(m_a: Vec<A>, mut f: Func) -> Vec<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> Vec<B>,
    {
        m_a.into_iter().flat_map(f).collect()
    }
}
```

### 4.2 Constrained Type: CausalTensor

```rust
// deep_causality_tensor/src/extensions/ext_hkt.rs

pub struct CausalTensorWitness;

impl HKT for CausalTensorWitness {
    type Constraint = TensorDataConstraint;
    type Type<T> = CausalTensor<T>
    where
        T: Satisfies<TensorDataConstraint>;
}

impl Functor<CausalTensorWitness> for CausalTensorWitness {
    fn fmap<A, B, Func>(fa: CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        A: Satisfies<TensorDataConstraint>,
        B: Satisfies<TensorDataConstraint>,
        Func: FnMut(A) -> B,
    {
        let shape = fa.shape().to_vec();
        let new_data: Vec<B> = fa.into_vec().into_iter().map(f).collect();
        CausalTensor::new(new_data, shape).unwrap()
    }
}

impl CoMonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Satisfies<TensorDataConstraint> + Clone,
    {
        fa.as_slice().first().cloned().expect("Non-empty tensor required")
    }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        A: Satisfies<TensorDataConstraint> + Clone,
        B: Satisfies<TensorDataConstraint>,
        Func: FnMut(&CausalTensor<A>) -> B,
    {
        let len = fa.len();
        let shape = fa.shape().to_vec();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let view = fa.shifted_view(i);
                f(&view)
            })
            .collect();
        CausalTensor::new(new_data, shape).unwrap()
    }
}
```

### 4.3 New Physics Type: CausalMultiField

```rust
// deep_causality_multivector/src/extensions/hkt_multifield.rs

pub struct CausalMultiFieldWitness<B: LinearAlgebraBackend>(PhantomData<B>);

impl<B: LinearAlgebraBackend> HKT for CausalMultiFieldWitness<B> {
    type Constraint = TensorDataConstraint;
    type Type<T> = CausalMultiField<B, T>
    where
        T: Satisfies<TensorDataConstraint>;
}

// Now CausalMultiField can participate in the full HKT hierarchy!
impl<B: LinearAlgebraBackend> Functor<CausalMultiFieldWitness<B>> for CausalMultiFieldWitness<B> {
    fn fmap<A, C, Func>(fa: CausalMultiField<B, A>, f: Func) -> CausalMultiField<B, C>
    where
        A: Satisfies<TensorDataConstraint>,
        C: Satisfies<TensorDataConstraint>,
        Func: FnMut(A) -> C,
    {
        // Implementation using backend operations
        todo!()
    }
}
```

---

## 5. Migration Plan

### 5.1 Traits to Replace

| Old Trait                      | New Trait               | Change                   |
|--------------------------------|-------------------------|--------------------------|
| `HKT`                          | `HKT`                   | Add `type Constraint`    |
| `HKT2`, `HKT3`, `HKT4`, `HKT5` | Unchanged               | Still needed for arity   |
| `Functor<F: HKT>`              | `Functor<F: HKT>`       | Add constraint bounds    |
| `Applicative<F: HKT>`          | `Applicative<F: HKT>`   | Add constraint bounds    |
| `Monad<F: HKT>`                | `Monad<F: HKT>`         | Add constraint bounds    |
| `CoMonad<F: HKT>`              | `CoMonad<F: HKT>`       | Add constraint bounds    |
| `BoundedComonad`               | **DELETED**             | Merged into `CoMonad`    |
| `Adjunction<L, R>`             | `Adjunction<L, R, Ctx>` | Add constraint bounds    |
| `BoundedAdjunction`            | **DELETED**             | Merged into `Adjunction` |

### 5.2 Implementation Migration

| Type                       | Add to impl                               |
|----------------------------|-------------------------------------------|
| `VecWitness`               | `type Constraint = NoConstraint;`         |
| `OptionWitness`            | `type Constraint = NoConstraint;`         |
| `BoxWitness`               | `type Constraint = NoConstraint;`         |
| `ResultWitness<E>`         | `type Constraint = NoConstraint;`         |
| `CausalTensorWitness`      | `type Constraint = TensorDataConstraint;` |
| `CausalMultiVectorWitness` | `type Constraint = NumericConstraint;`    |
| `ManifoldWitness`          | `type Constraint = NumericConstraint;`    |
| `TopologyWitness`          | `type Constraint = NumericConstraint;`    |
| `GraphWitness`             | `type Constraint = NumericConstraint;`    |
| `HypergraphWitness`        | `type Constraint = NumericConstraint;`    |
| `PointCloudWitness`        | `type Constraint = NumericConstraint;`    |
| `CsrMatrixWitness`         | `type Constraint = NumericConstraint;`    |
| `ChainWitness`             | `type Constraint = NumericConstraint;`    |

### 5.3 Deleted Traits

After migration, these traits are **removed entirely**:

- `BoundedComonad` → functionality merged into `CoMonad`
- `BoundedAdjunction` → functionality merged into `Adjunction`

---

## 6. Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)

1. Create `src/core/constraint.rs` with `Satisfies`, `NoConstraint`
2. Create `src/core/constraints.rs` with standard markers
3. Update `HKT` trait signature in `src/core/hkt.rs`
4. Update `Functor`, `Applicative`, `Monad` in `src/algebra/`
5. Replace `CoMonad` + `BoundedComonad` with unified `CoMonad`
6. Replace `Adjunction` + `BoundedAdjunction` with unified `Adjunction`

### Phase 2: Standard Extensions (Week 3)

1. Update `VecWitness`, `OptionWitness`, `BoxWitness`, etc. with `NoConstraint`
2. Add constraint bounds to all trait method calls
3. Verify all tests pass

### Phase 3: Domain Crates (Week 4-6)

1. `deep_causality_tensor`: Add `TensorDataConstraint`, update `CausalTensorWitness`
2. `deep_causality_multivector`: Update `CausalMultiVectorWitness`
3. `deep_causality_topology`: Update all topology witnesses
4. `deep_causality_sparse`: Update `CsrMatrixWitness`

### Phase 4: New Physics Types (Week 7-8)

1. Implement `CausalMultiFieldWitness<B>` with full HKT stack
2. Prepare for `SpinorManifoldWitness`, `GaugeManifoldWitness`

### Phase 5: Cleanup (Week 9-10)

1. Remove deprecated traits
2. Update all documentation
3. Run full test suite and benchmarks

---

## 7. Quality Assurance

### 7.1 Testing Requirements

| Category           | Target                       |
|--------------------|------------------------------|
| Unit Tests         | 100% coverage of new traits  |
| Property Tests     | Functor/Monad laws verified  |
| Integration Tests  | Cross-crate HKT usage        |
| Compile-Time Tests | Constraint violations caught |

### 7.2 Production Criteria

| Criterion      | Requirement                                          |
|----------------|------------------------------------------------------|
| Stability      | MSRV 1.75+                                           |
| Performance    | No regression in benchmarks                          |
| Documentation  | 100% public API documented                           |
| Legacy Removal | Zero traces of `BoundedComonad`, `BoundedAdjunction` |

---

## 8. Orphan Rule Handling

### 8.1 The Rule

**The crate defining the marker must define the blanket impl.**

### 8.2 Placement Guide

| Constraint             | Define In               |
|------------------------|-------------------------|
| `NoConstraint`         | `deep_causality_haft`   |
| `NumericConstraint`    | `deep_causality_haft`   |
| `ThreadSafeConstraint` | `deep_causality_haft`   |
| `CloneConstraint`      | `deep_causality_haft`   |
| `TensorDataConstraint` | `deep_causality_tensor` |
| `TensorDataThreadSafe` | `deep_causality_tensor` |

### 8.3 Decision Tree

```
Does constraint use only std traits + Zero/One/Field?
├── YES → Define in deep_causality_haft
└── NO → Define in crate that owns the required trait
```

---

## 9. Summary: Before vs After

### Before (Two Hierarchies)

```
HKT                          BoundedHKT (proposed, never built)
  └── Functor                  └── BoundedFunctor (proposed)
        └── Applicative              └── BoundedApplicative (proposed)
              └── Monad                   └── BoundedMonad (proposed)

CoMonad                      BoundedComonad ← HARDCODED BOUNDS
Adjunction                   BoundedAdjunction ← HARDCODED BOUNDS
```

### After (Unified)

```
HKT { type Constraint; type Type<T> where T: Satisfies<Constraint> }
  └── Functor<F: HKT>
        └── Applicative<F: HKT>
              └── Monad<F: HKT>

CoMonad<F: HKT>  ← Same trait for Vec and CausalTensor
Adjunction<L, R: HKT, Ctx>  ← Same trait for all adjunctions
```

**Result**: One trait hierarchy. All types. Zero hardcoded bounds.

---

## 10. References

1. `AGENTS.md` — Project conventions
2. `docs/HAFT.md` — Original HKT documentation
3. `docs/UNIFORM_MATH.md` — Mathematical foundations
4. Rust RFC 1598: Generic Associated Types
5. Haskell `ConstraintKinds` — Inspiration for `Satisfies` pattern
