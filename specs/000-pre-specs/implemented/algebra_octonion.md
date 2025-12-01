# Migration Guide: `Octonion` from `Num` to Algebraic Traits

## 1. Motivation

The current implementation of `Octonion` is based on the generic `Float` and `Num` traits. This is inconsistent with `Complex` and `Quaternion` which use the more specific and powerful algebraic traits from the `algebra` module. Migrating `Octonion` to this new system provides better static guarantees about its mathematical properties and aligns it with the rest of the number types in the crate.

This guide outlines the steps to refactor the `Octonion` implementation.

## 2. Key Changes

The core of the migration involves:
1.  Changing the generic bound from `F: Float` to `F: RealField`.
2.  Removing the dependency on the `Num` trait and the now-redundant `OctonionNumber` trait.
3.  Replacing them with direct implementations of the appropriate algebraic traits (`AbelianGroup`, `Distributive`, `DivisionAlgebra`).

## 3. Step-by-Step Migration Plan

### Step 1: Update Generic Bound to `RealField`

Everywhere `Octonion<F>` is defined or implemented, the generic bound `F: Float` must be changed to `F: RealField`. `RealField` provides all the necessary floating-point operations (`sin`, `cos`, `sqrt`, etc.) and also ensures the component type is a `Field`.

**Affected files:**
- `deep_causality_num/src/complex/octonion_number/mod.rs`
- `deep_causality_num/src/complex/octonion_number/arithmetic.rs`
- `deep_causality_num/src/complex/octonion_number/arithmetic_assign.rs`
- `deep_causality_num/src/complex/octonion_number/constructors.rs`
- `deep_causality_num/src/complex/octonion_number/octonion_number_impl.rs` (to be renamed/refactored)
- ... and all other files in the module.

**Example:**
```rust
// Before
pub struct Octonion<F> where F: Float { ... }
impl<F: Float> Add for Octonion<F> { ... }

// After
pub struct Octonion<F> where F: RealField { ... }
impl<F: RealField> Add for Octonion<F> { ... }
```

### Step 2: Remove `OctonionNumber` and `Num` Implementation

The `OctonionNumber` trait is a workaround for not using the `algebra` traits. It should be removed entirely.

1.  **Delete `OctonionNumber` trait:** In `deep_causality_num/src/complex/octonion_number/mod.rs`, delete the `pub trait OctonionNumber...` definition.
2.  **Remove `Num` implementation:** In the same file, delete `impl<F: Float> Num for Octonion<F> {}`.
3.  **Refactor `octonion_number_impl.rs`:**
    -   Rename `octonion_number_impl.rs` to a more suitable name like `ops.rs` or `methods.rs`.
    -   Change `impl<F: Float> OctonionNumber<F> for Octonion<F>` to a direct inherent implementation `impl<F: RealField> Octonion<F>`.
    -   Make all the methods within this block `pub` so they form the public API of the `Octonion` struct.

**Example (`octonion_number_impl.rs` -> `ops.rs`):**
```rust
// Before (in octonion_number_impl.rs)
use crate::complex::octonion_number::{Octonion, OctonionNumber};
use crate::float::Float;
impl<F: Float> OctonionNumber<F> for Octonion<F> {
    fn conjugate(&self) -> Self { ... }
    // ...
}

// After (in ops.rs)
use crate::complex::octonion_number::Octonion;
use crate::RealField;
impl<F: RealField> Octonion<F> {
    pub fn conjugate(&self) -> Self { ... }
    // ...
}
```

### Step 3: Implement Algebraic Traits

Create a new file `deep_causality_num/src/complex/octonion_number/algebra.rs` and add it to the `mod.rs` of the `octonion_number` module. In this file, implement the correct algebraic traits for `Octonion`.

#### 3.1. Marker Traits

Octonions are distributive, but **not** associative and **not** commutative.

```rust
// deep_causality_num/src/complex/octonion_number/algebra.rs

use crate::{Distributive, Octonion, RealField};

impl<T: RealField> Distributive for Octonion<T> {}

// DO NOT IMPLEMENT `Associative`
// DO NOT IMPLEMENT `Commutative`
```

#### 3.2. Group & Algebra Traits

Octonions form an `AbelianGroup` under addition and are a `DivisionAlgebra` over the real fields.

```rust
// deep_causality_num/src/complex/octonion_number/algebra.rs

use crate::{
    AbelianGroup, Distributive, DivisionAlgebra, MulGroup, Octonion, RealField,
};

// ... Marker trait impls from above ...

// Octonion addition is component-wise and commutative.
impl<T: RealField> AbelianGroup for Octonion<T> {}

// Octonions are a Division Algebra, but NOT associative.
// The blanket impl for `Algebra<T>` will apply, since we can satisfy its
// bounds: `Module<T> + Mul<...> + MulAssign + One + Distributive`.
impl<T: RealField> DivisionAlgebra<T> for Octonion<T> {
    fn conjugate(&self) -> Self {
        self.conjugate()
    }

    fn norm_sqr(&self) -> T {
        self.norm_sqr()
    }

    fn inverse(&self) -> Self {
        self.inverse()
    }
}
```
**Note**: For the `DivisionAlgebra` implementation to compile, `Octonion<T>` must implement `Module<T>`. This requires `MulAssign<T> for Octonion<T>` (scalar multiplication assignment), which is currently missing from `arithmetic_assign.rs`. This must be added.

**Action:** Add scalar `MulAssign` to `deep_causality_num/src/complex/octonion_number/arithmetic_assign.rs`:
```rust
impl<F: RealField + MulAssign> MulAssign<F> for Octonion<F> {
    fn mul_assign(&mut self, scalar: F) {
        self.s *= scalar;
        self.e1 *= scalar;
        self.e2 *= scalar;
        self.e3 *= scalar;
        self.e4 *= scalar;
        self.e5 *= scalar;
        self.e6 *= scalar;
        self.e7 *= scalar;
    }
}
```

## 4. Summary of Final Structure

After the migration, the `Octonion` implementation will align with `Complex` and `Quaternion`:
- It will be generic over `T: RealField`.
- It will have its core mathematical functions (`norm`, `inverse`, etc.) as public inherent methods.
- It will correctly implement the algebraic traits from the `algebra` module, statically declaring its properties (or lack thereof, e.g., associativity). Specifically, it will be a `DivisionAlgebra` but not an `AssociativeRing` or a `Field`.

This refactoring will improve code consistency, correctness, and allow `Octonion` to be used more effectively within the algebraic framework of the `deep_causality_num` crate.
