# HKT Fields: Resolving TensorData Bound Incompatibility

## Executive Summary

This document analyzes the fundamental incompatibility between the `TensorData` trait bound
required by `CausalMultiField` and the unconditional GAT required by haft's `HKT` trait.
We explore multiple solutions and provide a recommendation for enabling proper HKT trait
implementation for MultiField and derived types (SpinorManifold, GaugeManifold).

**Current Status:** `CausalMultiField` is not yet released, allowing for major design changes.

---

## 1. The Problem

### 1.1 The HKT Trait (haft)

```rust
// deep_causality_haft/src/core/hkt.rs
pub trait HKT {
    type Type<T>;  // NO BOUNDS on T
}
```

The haft HKT trait uses an **unconditional GAT** — `Type<T>` accepts any `T` with no constraints.
This is intentional: it allows maximum genericity for types like `Vec<T>`, `Option<T>`, etc.

### 1.2 The TensorData Bound

```rust
// deep_causality_tensor/src/traits/tensor_data.rs
pub trait TensorData: Field + Copy + Default + PartialOrd + Send + Sync + 'static {}
```

`TensorData` is a **rich trait** requiring:
- `Field` (algebraic operations: +, -, *, /, zero, one)
- `Copy` (value semantics, no heap allocation)
- `Default` (zero initialization)
- `PartialOrd` (comparison for max/min operations)
- `Send + Sync` (thread safety)
- `'static` (no borrowed data)

### 1.3 The Incompatibility

```rust
// CausalMultiField requires T: TensorData
pub struct CausalMultiField<B: LinearAlgebraBackend, T: TensorData> {
    data: B::Tensor<T>,  // Tensor<T> requires T: TensorData
    // ...
}

// Attempting to implement HKT
impl<B: LinearAlgebraBackend> HKT for CausalMultiFieldWitness<B> {
    type Type<T> = CausalMultiField<B, T>  // ERROR: need T: TensorData
    where
        T: TensorData;  // Compiler error: trait bounds not allowed in HKT::Type
}
```

**The core conflict:** HKT says "give me any T", but CausalMultiField says "T must be TensorData".

---

## 2. Analysis of Solutions

### Solution A: Add Bounded HKT Variants to Haft

**Idea:** Create parallel traits that accept bounds.

```rust
// New trait in haft
pub trait BoundedHKT<Bound> {
    type Type<T: Bound>;
}

// Usage
impl<B: LinearAlgebraBackend> BoundedHKT<TensorData> for CausalMultiFieldWitness<B> {
    type Type<T: TensorData> = CausalMultiField<B, T>;
}
```

**Pros:**
- Clean, explicit declaration of the bound
- Existing haft traits remain unchanged
- Type-level documentation of the constraint

**Cons:**
- Requires parallel trait hierarchies: `BoundedFunctor`, `BoundedMonad`, etc.
- Doubles the API surface of haft
- Users must choose which hierarchy to use
- Generic code can't work across both hierarchies

**Recommendation:** ⚠️ MODERATE — Significant API complexity trade-off.

---

### Solution B: Relax TensorData at the MultiField Type Level

**Idea:** Move the `TensorData` bound from the type definition to the impl blocks.

```rust
// BEFORE: Bound on type
pub struct CausalMultiField<B: LinearAlgebraBackend, T: TensorData> { ... }

// AFTER: No bound on type
pub struct CausalMultiField<B: LinearAlgebraBackend, T> {
    data: B::Tensor<T>,  // But Tensor<T> still requires TensorData...
}
```

**The Deeper Problem:** `B::Tensor<T>` is defined as:

```rust
pub trait LinearAlgebraBackend {
    type Tensor<T: TensorData>: ...;  // TensorData bound is here!
}
```

So the bound propagates from the backend trait, not just MultiField.

**Potential Sub-Solution B.1:** Relax the backend trait as well:

```rust
// Relaxed backend
pub trait LinearAlgebraBackend {
    type Tensor<T>;  // No bound here
    
    // Bounds move to methods
    fn create<T: TensorData>(data: &[T], shape: &[usize]) -> Self::Tensor<T>;
}
```

**Pros:**
- Enables HKT implementation
- More flexible type definitions

**Cons:**
- Major refactor of the entire tensor crate
- May lose some type safety (invalid Tensor<T> can exist)
- Performance: Some operations may panic at runtime instead of compile-time

**Recommendation:** ⚠️ HIGH RISK — Very invasive change across multiple crates.

---

### Solution C: Accept Inherent Methods as the Pattern

**Idea:** Don't implement HKT traits; provide equivalent functionality as inherent methods.

```rust
// Current approach
impl<B: LinearAlgebraBackend> CausalMultiFieldWitness<B> {
    pub fn fmap<A, NewT, F>(fa: CausalMultiField<B, A>, f: F) -> CausalMultiField<B, NewT>
    where
        A: TensorData + ...,
        NewT: TensorData + ...,
        F: FnMut(A) -> NewT,
    { ... }
}
```

**Pros:**
- Already works — no changes needed
- Full control over bounds
- Clear documentation of requirements

**Cons:**
- Can't write generic code over `Functor<F>` that works with MultiField
- No type-class style abstraction
- Duplication if many types follow this pattern

**Recommendation:** ✅ SAFE — Works today, minimal risk.

---

### Solution D: Wrapper Type with Phantom Data

**Idea:** Create a type that can hold any T but only allows operations when T: TensorData.

```rust
pub struct MultiFieldContainer<B: LinearAlgebraBackend, T> {
    // Store raw data without TensorData bound
    raw_data: Vec<T>,  // NOT B::Tensor<T>
    shape: [usize; 3],
    metric: Metric,
    dx: [T; 3],
    _backend: PhantomData<B>,
}

impl<B: LinearAlgebraBackend> HKT for MultiFieldContainerWitness<B> {
    type Type<T> = MultiFieldContainer<B, T>;  // Works! No bounds needed
}

// Conversion to actual CausalMultiField when T: TensorData
impl<B: LinearAlgebraBackend, T: TensorData> MultiFieldContainer<B, T> {
    pub fn to_causal_field(&self) -> CausalMultiField<B, T> {
        // Upload to backend tensor
        CausalMultiField::from_coefficients(...)
    }
}
```

**Pros:**
- HKT traits work for the container
- Conversion provides type safety
- Clear separation of concerns

**Cons:**
- Two types to manage (Container vs CausalMultiField)
- Conversion overhead (CPU → GPU upload)
- Confusing API: which type should users use?

**Recommendation:** ⚠️ MODERATE — Adds complexity but is viable.

---

### Solution E: Conditional HKT via Feature Flag (Compile-Time)

**Idea:** Use Rust's conditional compilation to enable HKT only when bounds are relaxed.

```rust
#[cfg(feature = "hkt_relaxed")]
impl<B: LinearAlgebraBackend> HKT for CausalMultiFieldWitness<B> {
    type Type<T> = CausalMultiField<B, T>;  // Requires relaxed TensorData
}

#[cfg(not(feature = "hkt_relaxed"))]
// Use inherent methods
```

**Pros:**
- User choice at compile time
- Backwards compatible

**Cons:**
- Doesn't actually solve the problem — still need relaxed bounds somewhere
- Feature flag complexity

**Recommendation:** ❌ AVOID — Kicks the can down the road.

---

### Solution F: New BoundedHKT in Haft with Functor/Monad Variants

**Idea:** Extend haft with a complete parallel hierarchy for bounded HKTs.

```rust
// haft additions
pub trait BoundedHKT {
    type Bound;
    type Type<T: Self::Bound>;
}

pub trait BoundedFunctor<F: BoundedHKT> {
    fn fmap<A, B, Func>(fa: F::Type<A>, f: Func) -> F::Type<B>
    where
        A: F::Bound,
        B: F::Bound,
        Func: FnMut(A) -> B;
}

// MultiField implementation
impl<B: LinearAlgebraBackend> BoundedHKT for CausalMultiFieldWitness<B> {
    type Bound = TensorData;
    type Type<T: TensorData> = CausalMultiField<B, T>;
}

impl<B> BoundedFunctor<CausalMultiFieldWitness<B>> for CausalMultiFieldWitness<B>
where B: LinearAlgebraBackend
{
    fn fmap<A, NewT, F>(fa: CausalMultiField<B, A>, f: F) -> CausalMultiField<B, NewT>
    where
        A: TensorData,
        NewT: TensorData,
        F: FnMut(A) -> NewT,
    {
        // Implementation
    }
}
```

**Pros:**
- Type-safe and explicit
- Generic programming over bounded HKTs
- Single Bound associated type documents the constraint

**Cons:**
- Significant haft expansion
- Learning curve for users
- May not compose well with unbounded HKT types

**Recommendation:** ✅ PREFERRED — Clean design if haft expansion is acceptable.

---

## 3. Recommendation

### Primary Recommendation: Solution F (BoundedHKT Hierarchy)

Given that:
1. `CausalMultiField` is not yet released (major changes possible)
2. SpinorManifold and GaugeManifold will have the same constraint
3. The pattern may apply to other tensor-backed types

**We recommend extending haft with a `BoundedHKT` hierarchy.**

### Implementation Plan

1. **Phase 1: Design BoundedHKT in haft**
   - Add `BoundedHKT` trait with associated `Bound` type
   - Add `BoundedFunctor`, `BoundedApplicative`, `BoundedMonad`, `BoundedComonad`
   - Ensure compatibility with existing unbounded hierarchy

2. **Phase 2: Implement for CausalMultiField**
   - Implement `BoundedHKT` with `Bound = TensorData`
   - Implement `BoundedFunctor`, etc.
   - Keep inherent methods as convenience wrappers

3. **Phase 3: Apply to SpinorManifold/GaugeManifold**
   - These types inherit the pattern from CausalMultiField
   - May add additional bounds (e.g., `Spinor` trait)

### Fallback: Solution C (Inherent Methods)

If haft expansion is not desired:
- Keep current inherent method approach
- Document clearly that HKT traits are not implemented
- Provide the same functionality through methods

---

## 4. Appendix: Type Signatures for BoundedHKT

```rust
// Core BoundedHKT trait
pub trait BoundedHKT {
    type Bound: ?Sized;
    type Type<T> where T: Self::Bound;
}

// BoundedFunctor
pub trait BoundedFunctor<F: BoundedHKT> {
    fn fmap<A, B, Func>(fa: F::Type<A>, f: Func) -> F::Type<B>
    where
        A: F::Bound,
        B: F::Bound,
        Func: FnMut(A) -> B;
}

// BoundedApplicative
pub trait BoundedApplicative<F: BoundedHKT>: BoundedFunctor<F> {
    fn pure<T>(value: T) -> F::Type<T>
    where
        T: F::Bound;
    
    fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
    where
        A: F::Bound + Clone,
        B: F::Bound,
        Func: FnMut(A) -> B + F::Bound;
}

// BoundedMonad
pub trait BoundedMonad<F: BoundedHKT>: BoundedApplicative<F> {
    fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        A: F::Bound,
        B: F::Bound,
        Func: FnMut(A) -> F::Type<B>;
}

// BoundedComonad
pub trait BoundedComonad<F: BoundedHKT>: BoundedFunctor<F> {
    fn extract<A>(fa: &F::Type<A>) -> A
    where
        A: F::Bound + Clone;
    
    fn extend<A, B, Func>(fa: &F::Type<A>, f: Func) -> F::Type<B>
    where
        A: F::Bound + Clone,
        B: F::Bound + Clone,
        Func: FnMut(&F::Type<A>) -> B;
}
```

---

## 5. Deep Dive: Bounded HKTs & GATs

### 5.1 Why GATs are the Key

Generic Associated Types (GATs) allow a trait to define a type constructor relative to `Self`.
For HKTs, this is the magic that maps `T` to `Container<T>`:

```rust
trait HKT {
    type Type<T>; // The GAT: A type constructor
}
```

Without GATs, we cannot express "A container that can hold *any* T" in a trait.

### 5.2 The "Arbitrary Bound" Problem

Currently, `BoundedComonad` in your codebase adds "random" bounds like logical patches:

```rust
// Current haft implementation
fn extend<A, B, Func>(...)
where
    A: Zero + Copy + Clone, // <--- Why Zero? Why Copy?
    B: Zero + Copy + Clone; // <--- Arbitrary!
```

This breaks universality. A `List<String>` is a valid Comonad, but `String` is not `Copy` or `Zero`.
So `List` cannot implement this `BoundedComonad`.

### 5.3 The Principled Solution: Bounded GATs

We want the **HKT implementor** to decide the bounds, not the trait definition.
However, Rust stable does not yet support "Generic Constraint Arguments" (e.g., `trait HKT<Bound>`).

**The Workaround: The Constraint Type Pattern**

We can encode bounds using a marker type and a helper trait.

```rust
// 1. A trait to check if T satisfies a meta-constraint C
pub trait Satisfies<C: ?Sized> {}

// 2. The Bounded HKT Trait
pub trait BoundedHKT {
    // The "meta-type" representing the constraint (e.g., TensorDataMarker)
    type Constraint: ?Sized;

    // The GAT, restricted to types satisfying the constraint
    type Type<T>: ?Sized; 
    
    // Note: We can't enforce `where T: ...` on the GAT declaration easily in stable,
    // so we enforce it on the *methods* of traits using BoundedHKT.
}

// 3. Functor using the bound
pub trait BoundedFunctor<F: BoundedHKT>: BoundedHKT {
    fn fmap<A, B, Func>(fa: Self::Type<A>, f: Func) -> Self::Type<B>
    where
        A: Satisfies<F::Constraint>, // A must satisfy F's bound
        B: Satisfies<F::Constraint>, // B must satisfy F's bound
        Func: FnMut(A) -> B;
}
```

**Practical Implementation for TensorData:**

```rust
// Define the marker
pub struct TensorDataConstraint;

// Implement checking logic
impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}

// Implement Witness
impl<B: Backend> BoundedHKT for CausalMultiFieldWitness<B> {
    type Constraint = TensorDataConstraint;
    type Type<T> = CausalMultiField<B, T>;
}
```

This removes "arbitrary" bounds from the library and moves them to the **Witness** where they belong.

---

## 6. Migration Strategy

This sketch outlines how to convert the existing codebase to this principled solution.

### Step 1: Lay the Foundation in `deep_causality_haft`

Create a new module `bounded` in `haft`:

1. Define `trait Satisfies<C>` (marker trait mechanism).
2. Define `trait BoundedHKT` with `type Constraint` and `type Type<T>`.
3. Port `Functor`, `Applicative`, `Monad` to `BoundedFunctor`, etc., using `where T: Satisfies<F::Constraint>`.

### Step 2: Define Constraints in `deep_causality_tensor`

1. Create `struct TensorDataConstraint;`.
2. `impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}`.

### Step 3: Implement for CausalMultiField

1. Implement `BoundedHKT` for `CausalMultiFieldWitness`.
   - `type Constraint = TensorDataConstraint;`
2. Implement `BoundedFunctor`, `BoundedMonad`, `BoundedComonad`.

### Step 4: Refactor Existing Bounded Traits

The current `BoundedComonad` (with `Zero + Copy`) is likely a specific instance of this general pattern.
1. Define `struct NumericConstraint;`
2. `impl<T: Zero + Copy> Satisfies<NumericConstraint> for T {}`
3. Refactor usages to use the generic `BoundedHKT` system instead of hardcoded bounds.

### Step 5: Update Topological Types

1. `SpinorManifold` implements `BoundedHKT` with `TensorDataConstraint`.
2. `GaugeManifold` implements `BoundedHKT` with `TensorDataConstraint`.

**Result:** A unified type system where `Vec<T>` (Unbounded), `CausalMultiField<T>` (TensorData), and `AudioBuffer<T>` (Copy) co-exist using the same Functor/Monad abstractions.
