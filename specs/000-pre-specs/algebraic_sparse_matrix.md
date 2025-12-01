# Algebraic Sparse Matrix Implementation Specification

## Overview

This specification details the implementation of algebraic traits from `deep_causality_num::algebra` for `CsrMatrix<T>` in the `deep_causality_sparse` crate. This is a **prerequisite** for implementing algebraic topology structures, particularly `Chain<T>` which uses `CsrMatrix<T>` for its sparse weight storage.

## Background

### Current State

The `CsrMatrix<T>` in `deep_causality_sparse/src/types/sparse_matrix/` currently has:

**Existing Operations** (in `ops.rs` and `api.rs`):
- ✅ `add_matrix(&self, other: &Self) -> Result<Self, SparseMatrixError>` (requires `T: Copy + Zero + PartialEq + Default`)
- ✅ `sub_matrix(&self, other: &Self) -> Result<Self, SparseMatrixError>` (requires `T: Copy + Sub<Output = T> + Zero + PartialEq + Default`)
- ✅ `scalar_mult(&self, scalar: T) -> Self` (requires `T: Copy + Mul<Output = T> + Zero + PartialEq + Default`)
- ✅ `transpose(&self) -> Self`
- ✅ `mat_mult(&self, other: &Self) -> Result<Self, SparseMatrixError>`
- ✅ `vec_mult(&self, x: &[T]) -> Result<Vec<T>, SparseMatrixError>`

**Missing**:
- ❌ `Zero`, `One` trait implementations
- ❌ `AbelianGroup`, `AddGroup` trait implementations
- ❌ `Module<S>` trait implementation
- ❌ `Ring` trait implementations
- ❌ Standard operator overloads (`std::ops::{Add, Sub, Mul, Neg}`)
- ❌ Algebraic methods with panic-based error handling (for trait compatibility)

**Empty File**:
- `algebra.rs` exists but contains only header comments (6 lines)

### Motivation

`Chain<T>` in `deep_causality_topology` uses `CsrMatrix<T>` for sparse weight storage:
```rust
pub struct Chain<T> {
    pub(crate) complex: Arc<SimplicialComplex>,
    pub(crate) grade: usize,
    pub(crate) weights: CsrMatrix<T>, // ← Needs algebraic traits
}
```

To implement **homological algebra** operations on chains:
- `Chain + Chain` requires `CsrMatrix` to implement `Add`
- Scalar multiplication `s * Chain` requires `CsrMatrix` to implement `Module<S>`
- Abstract algebraic properties require `AbelianGroup`, `Ring` traits

## Design Challenges

### Challenge 1: Shape-Dependent Zero

**Problem**: `CsrMatrix` needs shape information to create a zero matrix, but the `Zero` trait requires `fn zero() -> Self` with no parameters.

**Solution Options**:
1. **Skip `Zero` trait** (RECOMMENDED): Provide `CsrMatrix::zero(rows, cols)` method instead
2. Implement `Zero` to return `(0, 0)` empty matrix (loses shape information)
3. Add a type-level shape parameter (too complex)

**Decision**: Use Option 1 - provide `zero(rows, cols)` method, skip `Zero` trait.

**Rationale**:
- `Chain<T>` knows the shape (from `complex.skeletons[grade].simplices.len()`)
- Other types (Tensor, MultiVector) also faced this and used shape-dependent constructors
- Mathematical correctness: zero matrix is shape-dependent

### Challenge 2: Result vs. Panic

**Problem**: Current operations return `Result<Self, SparseMatrixError>`, but algebraic traits require `Self` output (no `Result`).

**Solution**:
1. Create **new algebraic methods** (`add`, `sub`, `scale`) that panic on error
2. Keep **existing API methods** (`add_matrix`, `sub_matrix`, `scalar_mult`) returning `Result`
3. Implement **standard operators** to call algebraic methods (panic-based)

**Example**:
```rust
// Existing (safe, Result-based)
pub fn add_matrix(&self, other: &Self) -> Result<Self, SparseMatrixError> { /* ... */ }

// New (algebraic, panic-based)
pub fn add(&self, rhs: &Self) -> Self {
    self.add_matrix_impl(rhs).expect("Shape mismatch in CsrMatrix addition")
}

// Standard operator (calls algebraic method)
impl<T> Add for CsrMatrix<T> where T: /* ... */ {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { (&self).add(&rhs) }
}
```

### Challenge 3: Generic Bounds

**Problem**: Different algebraic operations require different bounds on `T`.

**Solution**: Use **tiered implementation** pattern from `CausalMultiVector`:

**Tier 1: Additive Group** (requires `T: AbelianGroup + Copy`)
- `zero(rows, cols)`
- `add(&self, rhs: &Self) -> Self`
- `sub(&self, rhs: &Self) -> Self`
- `neg(&self) -> Self`

**Tier 2: Module** (requires `T: Module<S> + Copy`, `S: Ring + Copy`)
- `scale<S>(&self, scalar: S) -> Self`

**Tier 3: Multiplicative** (requires `T: Ring + Copy`)
- `one(size)` (identity matrix)
- `mul(&self, rhs: &Self) -> Self` (matrix multiplication)

### Challenge 4: CSR Format Preservation

**Constraint**: All operations must maintain CSR invariants:
- `row_indices` is sorted and cumulative
- `col_indices` are sorted within each row
- No explicit zeros in `values`

**Verification**: Existing `*_impl` functions already handle this correctly.

## Proposed Implementation

### Directory Structure

```
deep_causality_sparse/src/types/sparse_matrix/
├── algebra/
│   ├── mod.rs        [NEW] - Re-export modules
│   ├── group.rs      [NEW] - Additive group operations
│   ├── module.rs     [NEW] - Scalar multiplication
│   ├── ring.rs       [NEW] - Multiplicative operations
│   └── traits.rs     [NEW] - Trait implementations
├── arithmetic/
│   └── mod.rs        [NEW] - Standard operator overloads
├── algebra.rs        [DELETE or RENAME] - Currently empty placeholder
├── api.rs            [KEEP] - Public Result-based API
├── ops.rs            [KEEP] - Implementation functions (`*_impl`)
├── mod.rs            [MODIFY] - Add algebra module
└── ... (other files unchanged)
```

### Tier 1: Additive Group (Priority: CRITICAL)

**File**: `algebra/group.rs`

```rust
use crate::{CsrMatrix, SparseMatrixError};
use deep_causality_num::{AbelianGroup, AddGroup, Zero};

impl<T> CsrMatrix<T>
where
    T: AbelianGroup + Copy,
{
    /// Creates a zero matrix with the given shape.
    /// 
    /// # Arguments
    /// * `rows` - Number of rows
    /// * `cols` - Number of columns
    /// 
    /// # Returns
    /// A sparse matrix with all elements zero (empty CSR structure).
    pub fn zero(rows: usize, cols: usize) -> Self {
        Self {
            row_indices: vec![0; rows + 1],
            col_indices: Vec::new(),
            values: Vec::new(),
            shape: (rows, cols),
        }
    }

    /// Element-wise matrix addition (panics on shape mismatch).
    /// 
    /// # Panics
    /// Panics if `self.shape != rhs.shape`.
    pub fn add(&self, rhs: &Self) -> Self {
        self.add_matrix_impl(rhs)
            .expect("CsrMatrix shape mismatch in add")
    }

    /// Element-wise matrix subtraction (panics on shape mismatch).
    /// 
    /// # Panics
    /// Panics if `self.shape != rhs.shape`.
    pub fn sub(&self, rhs: &Self) -> Self
    where
        T: std::ops::Sub<Output = T>,
    {
        self.sub_matrix_impl(rhs)
            .expect("CsrMatrix shape mismatch in sub")
    }

    /// Element-wise negation.
    pub fn neg(&self) -> Self
    where
        T: std::ops::Neg<Output = T>,
    {
        Self {
            row_indices: self.row_indices.clone(),
            col_indices: self.col_indices.clone(),
            values: self.values.iter().map(|&v| -v).collect(),
            shape: self.shape,
        }
    }
}
```

### Tier 2: Module (Priority: HIGH)

**File**: `algebra/module.rs`

```rust
use crate::CsrMatrix;
use deep_causality_num::{Module, Ring};

impl<T> CsrMatrix<T> {
    /// Scalar multiplication.
    /// 
    /// # Arguments
    /// * `scalar` - The scalar to multiply by
    /// 
    /// # Returns
    /// A new matrix where each element is multiplied by `scalar`.
    pub fn scale<S>(&self, scalar: S) -> Self
    where
        T: Module<S> + Copy,
        S: Ring + Copy,
    {
        // For CsrMatrix, scalar multiplication is element-wise
        // T: Module<S> implies T: Mul<S, Output = T>
        let new_values: Vec<T> = self.values.iter().map(|&v| v * scalar).collect();
        Self {
            row_indices: self.row_indices.clone(),
            col_indices: self.col_indices.clone(),
            values: new_values,
            shape: self.shape,
        }
    }
}
```

### Tier 3: Ring (Priority: MEDIUM)

**File**: `algebra/ring.rs`

```rust
use crate::CsrMatrix;
use deep_causality_num::{Ring, One};

impl<T> CsrMatrix<T>
where
    T: Ring + Copy,
{
    /// Creates an identity matrix of size `n × n`.
    /// 
    /// # Arguments
    /// * `size` - The dimension of the square identity matrix
    /// 
    /// # Returns
    /// An identity matrix where `I[i,i] = 1` and `I[i,j] = 0` for `i != j`.
    pub fn one(size: usize) -> Self {
        let mut row_indices = vec![0; size + 1];
        let mut col_indices = Vec::with_capacity(size);
        let mut values = Vec::with_capacity(size);

        for i in 0..size {
            col_indices.push(i);
            values.push(T::one());
            row_indices[i + 1] = i + 1;
        }

        Self {
            row_indices,
            col_indices,
            values,
            shape: (size, size),
        }
    }

    /// Matrix multiplication (panics on dimension mismatch).
    /// 
    /// Computes `self * rhs` using sparse matrix multiplication.
    /// 
    /// # Panics
    /// Panics if `self.cols != rhs.rows`.
    pub fn mul(&self, rhs: &Self) -> Self {
        self.mat_mult_impl(rhs)
            .expect("CsrMatrix dimension mismatch in mul")
    }
}
```

### Tier 4: Standard Operators (Priority: HIGH)

**File**: `arithmetic/mod.rs`

```rust
use crate::CsrMatrix;
use deep_causality_num::AbelianGroup;
use std::ops::{Add, Sub, Mul, Neg};

// Add for owned values
impl<T> Add for CsrMatrix<T>
where
    T: AbelianGroup + Copy,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        (&self).add(&rhs)
    }
}

// Add for borrowed values
impl<T> Add for &CsrMatrix<T>
where
    T: AbelianGroup + Copy,
{
    type Output = CsrMatrix<T>;
    fn add(self, rhs: Self) -> CsrMatrix<T> {
        self.add(rhs)
    }
}

// Add for mixed ownership (4 variants total)
impl<T> Add<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: AbelianGroup + Copy,
{
    type Output = Self;
    fn add(self, rhs: &Self) -> Self {
        (&self).add(rhs)
    }
}

impl<T> Add<CsrMatrix<T>> for &CsrMatrix<T>
where
    T: AbelianGroup + Copy,
{
    type Output = CsrMatrix<T>;
    fn add(self, rhs: CsrMatrix<T>) -> CsrMatrix<T> {
        self.add(&rhs)
    }
}

// Similar patterns for Sub, Mul, Neg
// (Implementation follows same structure as Add)
```

### Tier 5: Trait Implementations (Priority: MEDIUM)

**File**: `algebra/traits.rs`

```rust
use crate::CsrMatrix;
use deep_causality_num::{AbelianGroup, AddGroup, Zero, One};

// NOTE: Do NOT implement Zero trait (shape-dependent)
// Use CsrMatrix::zero(rows, cols) instead

impl<T> AbelianGroup for CsrMatrix<T>
where
    T: AbelianGroup + Copy + std::ops::Neg<Output = T>,
{
    // Marker trait, automatically satisfied
}

// AddGroup is automatically implemented via blanket impl in deep_causality_num
// because CsrMatrix implements Add, Sub, Neg, Clone

// NOTE: Do NOT implement One trait (size-dependent for identity matrix)
// Use CsrMatrix::one(size) instead for square identity matrices

// Module is automatically implemented via blanket impl in deep_causality_num
// when CsrMatrix<T> implements AbelianGroup and Mul<S>
```

## Implementation Phases

### Phase 1: Directory Setup and Group Operations (Week 1)
- [ ] Create `algebra/` directory
- [ ] Create `algebra/mod.rs` with module exports
- [ ] Implement `algebra/group.rs` (zero, add, sub, neg)
- [ ] Add unit tests for group operations
- [ ] Verify with `Chain<f64>` addition

**Success Criteria**:
```rust
let a = CsrMatrix::zero(3, 3);
let b = CsrMatrix::one(3);
let c = &a + &b; // Should compile and work
assert_eq!(c.shape(), (3, 3));
```

### Phase 2: Module and Scalar Operations (Week 1)
- [ ] Implement `algebra/module.rs` (scale)
- [ ] Add unit tests for scalar multiplication
- [ ] Verify with `Chain<Complex<f64>>` scaling

**Success Criteria**:
```rust
let a = CsrMatrix::one(3);
let b = a.scale(2.0);
assert_eq!(b.get_value_at(0, 0), 2.0);
```

### Phase 3: Standard Operators (Week 2)
- [ ] Create `arithmetic/` directory
- [ ] Implement `arithmetic/mod.rs` (Add, Sub, Mul, Neg)
- [ ] Add comprehensive operator tests
- [ ] Verify all ownership combinations work

**Success Criteria**:
```rust
let a = CsrMatrix::one(3);
let b = CsrMatrix::one(3);
let c = a + b;           // owned + owned
let d = &c + &c;         // ref + ref
let e = c + &d;          // owned + ref
let f = &d + e;          // ref + owned
```

### Phase 4: Ring Operations (Week 2)
- [ ] Implement `algebra/ring.rs` (one, mul)
- [ ] Add matrix multiplication tests
- [ ] Verify identity matrix properties

**Success Criteria**:
```rust
let i = CsrMatrix::one(3);
let a = /* some matrix */;
let b = &i * &a;
assert_eq!(a, b); // I * A = A
```

### Phase 5: Trait Implementations (Week 3)
- [ ] Implement `algebra/traits.rs`
- [ ] Verify blanket implementations work
- [ ] Add composition tests with `Chain<T>`

**Success Criteria**:
```rust
fn test_group<T: AbelianGroup>(x: T, y: T) -> T {
    x + y
}
let a = CsrMatrix::one(3);
let b = CsrMatrix::one(3);
let c = test_group(a, b); // Should compile
```

### Phase 6: Integration and Documentation (Week 3)
- [ ] Update `mod.rs` to re-export algebra
- [ ] Add comprehensive documentation
- [ ] Create examples in `examples/` directory
- [ ] Update `README.md`

**Success Criteria**:
- All tests pass
- `cargo doc` builds without warnings
- Examples demonstrate algebraic composition

## Verification Plan

### Unit Tests

**File**: `deep_causality_sparse/tests/algebra_tests.rs`

```rust
use deep_causality_sparse::CsrMatrix;
use deep_causality_num::{Zero, AbelianGroup};

#[test]
fn test_zero_matrix() {
    let z: CsrMatrix<f64> = CsrMatrix::zero(3, 3);
    assert_eq!(z.shape(), (3, 3));
    assert_eq!(z.values().len(), 0); // No non-zero elements
}

#[test]
fn test_additive_identity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let z = CsrMatrix::zero(2, 2);
    let b = &a + &z;
    assert_eq!(a, b); // A + 0 = A
}

#[test]
fn test_additive_inverse() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let neg_a = a.neg();
    let z = &a + &neg_a;
    assert_eq!(z.values().len(), 0); // A + (-A) = 0
}

#[test]
fn test_commutativity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(1, 1, 2.0)]).unwrap();
    let ab = &a + &b;
    let ba = &b + &a;
    assert_eq!(ab, ba); // A + B = B + A
}

#[test]
fn test_associativity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(1, 1, 2.0)]).unwrap();
    let c = CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0)]).unwrap();
    let ab_c = (&a + &b) + &c;
    let a_bc = &a + (&b + &c);
    assert_eq!(ab_c, a_bc); // (A + B) + C = A + (B + C)
}

#[test]
fn test_scalar_multiplication() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let b = a.scale(3.0);
    assert_eq!(b.get_value_at(0, 0), 3.0);
    assert_eq!(b.get_value_at(1, 1), 6.0);
}

#[test]
fn test_matrix_multiplication_identity() {
    let i = CsrMatrix::one(3);
    let a = CsrMatrix::from_triplets(3, 3, &[(0, 0, 1.0), (1, 1, 2.0), (2, 2, 3.0)]).unwrap();
    let b = i.mul(&a);
    assert_eq!(a, b); // I * A = A
}

#[test]
#[should_panic(expected = "shape mismatch")]
fn test_add_shape_mismatch() {
    let a = CsrMatrix::zero(2, 2);
    let b = CsrMatrix::zero(3, 3);
    let _c = a + b; // Should panic
}
```

### Integration Tests with Chain

**File**: `deep_causality_topology/tests/chain_sparse_matrix_tests.rs`

```rust
use deep_causality_topology::Chain;
use deep_causality_sparse::CsrMatrix;
use deep_causality_num::Complex;

#[test]
fn test_chain_addition() {
    // Create two chains and add them
    // This tests that CsrMatrix algebraic traits work with Chain
    let complex = /* ... */;
    let w1 = CsrMatrix::from_triplets(5, 1, &[(0, 0, 1.0), (2, 0, 0.5)]).unwrap();
    let w2 = CsrMatrix::from_triplets(5, 1, &[(1, 0, 2.0), (3, 0, 1.0)]).unwrap();
    
    let c1 = Chain::new(complex.clone(), 1, w1);
    let c2 = Chain::new(complex.clone(), 1, w2);
    
    let c3 = c1 + c2; // Should use CsrMatrix::add internally
    // Verify c3.weights is correct
}

#[test]
fn test_chain_with_complex_coefficients() {
    // Test Chain<Complex<f64>> to verify algebraic composition
    let complex = /* ... */;
    let z1 = Complex::new(1.0, 0.5);
    let z2 = Complex::new(0.5, 1.0);
    
    let w = CsrMatrix::from_triplets(3, 1, &[(0, 0, z1), (1, 0, z2)]).unwrap();
    let chain = Chain::new(complex, 1, w);
    
    let scaled = chain.scale(Complex::new(2.0, 0.0));
    // Verify complex scalar multiplication works
}
```

## Benefits

### 1. Enables Homological Algebra
```rust
// Chain addition
let boundary = c1.boundary(); // ∂c₁
let sum_chain = c1 + c2;      // c₁ + c₂
let boundary_sum = sum_chain.boundary(); // ∂(c₁ + c₂)

// Verify: ∂(c₁ + c₂) = ∂c₁ + ∂c₂ (boundary is a homomorphism)
assert_eq!(boundary_sum, boundary1 + boundary2);
```

### 2. Enables Algebraic Composition
```rust
// Chains with complex coefficients (quantum topology)
let chain: Chain<Complex<f64>> = /* ... */;

// Chains with multivector coefficients (geometric algebra)
let chain: Chain<CausalMultiVector<f64>> = /* ... */;

// Sparse matrices with quaternion entries (non-commutative geometry)
let mat: CsrMatrix<Quaternion<f64>> = /* ... */;
```

### 3. Type-Safe Linear Algebra
```rust
fn homology_rank<T: Field>(boundary: &CsrMatrix<T>) -> usize {
    // Compute rank using Gaussian elimination
    // Type safety ensures only Field types (with division) are used
}
```

## Risks and Mitigations

> [!WARNING]
> **Performance Risk**: Panic-based error handling may be slower than Result-based.
> **Mitigation**: Keep both APIs - `add_matrix` (Result) and `add` (panic) for different use cases.

> [!WARNING]
> **Breaking Change Risk**: Adding operator overloads changes API surface.
> **Mitigation**: This is a new feature, not changing existing API. Semver: minor version bump.

> [!CAUTION]
> **Shape Mismatch Panics**: Operators will panic on shape mismatch.
> **Mitigation**: Document clearly. Users should use Result-based API if they want error handling.

## Alternative Designs Considered

### Alternative 1: Shape in Type System

```rust
struct CsrMatrix<T, const ROWS: usize, const COLS: usize> { /* ... */ }
```

**Pros**: Compile-time shape checking
**Cons**: 
- Cannot use with dynamic sizes (runtime simplicial complexes)
- Massive API complexity
- Generic const expressions still unstable in Rust

**Decision**: **REJECTED** - Too complex, not practical for topology use cases.

### Alternative 2: Implement Zero Trait with (0, 0) Matrix

```rust
impl<T: Zero> Zero for CsrMatrix<T> {
    fn zero() -> Self {
        Self::zero(0, 0) // Empty matrix
    }
}
```

**Pros**: Satisfies trait requirements
**Cons**: 
- Loses shape information
- `a + CsrMatrix::zero()` would panic (shape mismatch)
- Not mathematically correct (zero should preserve shape)

**Decision**: **REJECTED** - Mathematically incorrect.

### Alternative 3: Always Unwrap in Operators

Instead of creating separate `add()` and `add_matrix()`:

```rust
impl Add for CsrMatrix<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.add_matrix(&rhs).unwrap() // Always unwrap
    }
}
```

**Pros**: Simpler API (only one method)
**Cons**: 
- No safe API for users who want error handling
- Panics without clear error messages

**Decision**: **REJECTED** - Need both safe and algebraic APIs.

## Dependencies

### Existing Dependencies
- `deep_causality_num` - Provides algebraic traits

### No New Dependencies Required
- All operations use existing `*_impl` functions
- Only adding trait implementations and wrapper methods

## Summary

This specification enables `CsrMatrix<T>` to participate in algebraic composition by:
1. Providing algebraic methods (`add`, `sub`, `neg`, `scale`, `mul`)
2. Implementing standard operators (`Add`, `Sub`, `Mul`, `Neg`)
3. Satisfying algebraic traits (`AbelianGroup`, `Module<S>`)
4. Maintaining backward compatibility (keeping Result-based API)

The implementation follows proven patterns from `CausalTensor` and `CausalMultiVector`, ensuring type-safe homological algebra for `Chain<T>` and other topological structures.
