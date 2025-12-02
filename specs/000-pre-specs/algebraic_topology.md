# Algebraic Topology Implementation Plan

## Overview

This specification outlines the implementation of algebraic traits from `deep_causality_num::algebra` for all topology types in `deep_causality_topology`. The goal is to enable:
1. **Algebraic Composition**: Allow topologies to compose with `CausalTensor`, `CausalMultiVector`, and complex numbers
2. **Differential Structures**: Enable `Field` trait bounds for differential geometry, differential forms, and field theory applications

## Background

### Algebraic Trait Hierarchy

The `deep_causality_num` algebraic traits form a hierarchy:

```
Magma (Mul/Add operation)
  ↓
Monoid (Magma + Identity)
  ↓
Group (Monoid + Inverse)
  ↓
AbelianGroup (Group + Commutative)
  ↓
Ring (AbelianGroup + MulMonoid + Distributive)
  ↓
CommutativeRing (Ring + Commutative Mul)
  ↓
Field (CommutativeRing + Multiplicative Inverse)
  ↓
RealField (Field + Ordering + Transcendentals)
```

**Additional Structures:**
- `Module<S>`: Vector space over scalar ring S (requires `AbelianGroup + Mul<S>`)
- `AssociativeRing`: Ring with associative multiplication (for non-commutative algebras like Quaternions)

### Implementation Pattern (from Tensor/MultiVector)

The "indirect" implementation strategy used for `CausalTensor` and `CausalMultiVector`:
1. Implement algebraic **methods** directly on the struct with appropriate generic bounds
2. Implement **standard operators** (`std::ops::{Add, Sub, Mul, Div, Neg}`) by calling these methods
3. Implement **algebraic traits** where applicable, leveraging blanket implementations

## Topology Structures Analysis

### 1. Chain
**Structure**: Weighted collection of simplices (sparse vector over simplicial complex)
- `complex: Arc<SimplicialComplex>`
- `grade: usize` (dimension)
- `weights: CsrMatrix<T>` (sparse weights)

**Algebraic Semantics**: Chain is a **vector in a chain complex** (homological algebra)
- Addition: Formal sum of chains
- Scalar multiplication: Scale weights
- Zero: Empty chain (all weights zero)

**Applicable Traits**:
- ✅ `AbelianGroup` (chain addition is commutative)
- ✅ `Module<S>` (chains form a module over scalar ring S)
- ❌ `Ring` (no natural chain multiplication)
- ✅ Boundary operator `∂: Chain[k] → Chain[k-1]` (separate method)

### 2. Graph
**Structure**: Nodes with adjacency list and cursor (comonadic focus)
- `num_vertices: usize`
- `adjacencies: BTreeMap<usize, Vec<usize>>`
- `data: CausalTensor<T>` (node metadata)
- `cursor: usize` (comonadic focus)

**Algebraic Semantics**: Graph is **NOT naturally algebraic** (graphs don't add/multiply directly)
- The `data` tensor already has algebraic structure
- Graph operations are topological (add edge, traverse)

**Applicable Traits**:
- ❌ Direct algebraic traits (graph structure itself)
- ✅ Algebraic operations on `data` field (already via `CausalTensor<T>`)
- 📝 **Consider**: Graph Laplacian, Adjacency Matrix as derived algebraic structures

### 3. Hypergraph
**Structure**: Generalization of graphs with hyperedges
- Similar to `Graph`, stores multi-ary relations
- `data: CausalTensor<T>`

**Algebraic Semantics**: Same as Graph
- ❌ Direct algebraic traits
- ✅ Algebraic operations via `data` field

### 4. Manifold
**Structure**: Simplicial complex satisfying manifold properties + data field
- `complex: SimplicialComplex`
- `data: CausalTensor<T>` (field values on simplices)
- `cursor: usize`

**Algebraic Semantics**: Manifold itself is **geometric**, but **fields on manifolds** are algebraic
- The `data` represents a **differential form** or **scalar/vector field**
- Addition/multiplication of fields is point-wise

**Applicable Traits**:
- ❌ Manifold structure itself
- ✅ Operations on `data` field (already via `CausalTensor<T>`)
- 📝 **Key Insight**: With `T: Field`, `data` becomes a **differential field**
- 📝 **Future**: Exterior derivative `d`, Hodge star `⋆` (requires geometric structure)

### 5. PointCloud
**Structure**: Collection of points in d-dimensional space
- `points: CausalTensor<f64>` (coordinates)
- `metadata: CausalTensor<T>`
- `cursor: usize`

**Algebraic Semantics**: Point clouds are **data structures**, not algebraic objects
- Operations: k-NN, triangulation, clustering (geometric/topological)

**Applicable Traits**:
- ❌ Direct algebraic traits
- ✅ Algebraic operations via `metadata` field

### 6. SimplicialComplex
**Structure**: Collection of simplices with boundary operators
- `skeletons: Vec<Skeleton>` (geometric entities)
- `boundary_operators: Vec<CsrMatrix<i8>>` (∂)
- `coboundary_operators: Vec<CsrMatrix<i8>>` (∂*)

**Algebraic Semantics**: The complex itself is **combinatorial**, but **chain groups** are algebraic
- `C_k = ℤ^{num_k_simplices}` (free abelian group)
- Boundary: `∂_k: C_k → C_{k-1}` (group homomorphism)

**Applicable Traits**:
- ❌ Complex structure itself
- ✅ **Create `ChainGroup` type** that implements algebraic traits
- 📝 `Chain<T>` already exists and should implement these traits

### 7. Simplex
**Structure**: Single geometric simplex (not a collection)
- Individual simplex representation

**Algebraic Semantics**: Not applicable
- ❌ Single geometric object

### 8. Skeleton
**Structure**: Collection of all k-simplices
- `simplices: Vec<Simplex>`

**Algebraic Semantics**: Storage structure, not algebraic
- ❌ Direct algebraic traits

### 9. Topology (Base)
**Structure**: General topology container with topological invariants
- Stores Euler characteristic, Betti numbers
- `data: CausalTensor<T>`

**Algebraic Semantics**: Similar to Manifold
- ❌ Topology structure itself
- ✅ Operations on `data` field

## Implementation Strategy

### Priority 1: Chain

**Rationale**: Chains are the **fundamental algebraic objects** in algebraic topology.

**Implementation**:

1. **Create `algebra` module**: `deep_causality_topology/src/types/chain/algebra/`
   - `group.rs`: Implement `zero`, `add`, `sub`, `neg`
   - `module.rs`: Implement `scale`
   - `traits.rs`: Implement `AbelianGroup`, `Module<S>`
   - **Context Handling**: Ensure methods support explicit zero values for types like `CausalMultiVector` that don't implement `Zero`.

2. **Method Implementations** (`group.rs`):
```rust
impl<T> Chain<T>
where
    T: AbelianGroup + Copy,
{
    pub fn zero(complex: Arc<SimplicialComplex>, grade: usize) -> Self {
        let size = complex.skeletons[grade].simplices.len();
        // Uses CsrMatrix::zero if T: Zero, or requires explicit zero construction
        // Note: If T does not implement Zero, we need a way to pass the zero value or use a default.
        // For now, we assume T: Zero for the default zero() constructor, 
        // but allow manual construction for non-Zero types.
        let weights = CsrMatrix::zero(1, size); 
        Self { complex, grade, weights }
    }

    pub fn add(&self, rhs: &Self) -> Self {
        assert_eq!(self.grade, rhs.grade, "Grade mismatch");
        assert!(Arc::ptr_eq(&self.complex, &rhs.complex), "Complex mismatch");
        
        let weights = &self.weights + &rhs.weights; // CsrMatrix addition
        Self {
            complex: Arc::clone(&self.complex),
            grade: self.grade,
            weights,
        }
    }

    pub fn sub(&self, rhs: &Self) -> Self { /* similar */ }
    pub fn neg(&self) -> Self { /* negate weights */ }
}
```

3. **Module Implementation** (`module.rs`):
```rust
impl<T> Chain<T> {
    pub fn scale<S>(&self, scalar: S) -> Self
    where
        T: Module<S> + Copy,
        S: Ring + Copy,
    {
        let weights = &self.weights * scalar; // Scalar multiplication
        Self {
            complex: Arc::clone(&self.complex),
            grade: self.grade,
            weights,
        }
    }
}
```

4. **Standard Operators** (in `arithmetic/mod.rs`):
```rust
impl<T> Add for Chain<T>
where
    T: AbelianGroup + Copy,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self { (&self).add(&rhs) }
}
// Similar for Sub, Neg
```

5. **Trait Implementations** (`traits.rs`):
```rust
use deep_causality_num::{Zero, One, AbelianGroup};

impl<T> Zero for Chain<T>
where
    T: Zero + Copy,
{
    fn zero() -> Self {
        // Problem: Need complex and grade!
        // Solution: Use shape-dependent constructor or skip Zero trait
        panic!("Chain::zero() requires complex and grade, use Chain::zero(complex, grade)")
    }
    fn is_zero(&self) -> bool { self.weights.is_zero() }
}

impl<T> AbelianGroup for Chain<T>
where
    T: AbelianGroup + Copy,
{
    // Marker trait, automatically satisfied
}
```

> [!IMPORTANT]
> **Design Decision**: `Chain` cannot implement `Zero` trait directly (needs complex+grade).
> Instead, provide `Chain::zero(complex, grade)` method. This is acceptable.

### Priority 2: Differential Fields on Manifolds

**Rationale**: Enabling `Field` trait on `Manifold<T>` data unlocks **differential geometry**.

**Implementation**:

1. **Update `Manifold<T>` to accept `Field` types**:
```rust
// Existing bound
impl<T> Manifold<T>
where
    T: Default + Copy + Clone + PartialEq + Zero,
{ /* ... */ }

// Add Field-specific methods
impl<T> Manifold<T>
where
    T: Field + Copy,
{
    /// Compute the Laplacian of the field (Δf)
    /// Uses the boundary operators and Hodge star
    pub fn laplacian(&self) -> CausalTensor<T> {
        // Δ = d*d + dd* (Laplace-Beltrami operator)
        // Implementation requires:
        // 1. Coboundary operator (d)
        // 2. Hodge star (⋆)
        // 3. Field multiplication
        todo!("Implement using boundary operators from complex")
    }

    /// Exterior derivative (d: Ω^k → Ω^{k+1})
    pub fn exterior_derivative(&self, k: usize) -> CausalTensor<T> {
        // d(f) = ∂f (for 0-forms, i.e., scalar fields)
        // Implementation uses boundary operator matrices
        todo!("Implement using boundary operators")
    }
}
```

2. **No new algebraic trait implementations needed**:
   - `data: CausalTensor<T>` already has all algebraic traits
   - Just add **geometric operators** that leverage `Field` bound

### Priority 3: Enhanced CsrMatrix Algebraic Support

**Rationale**: `CsrMatrix<T>` (used in `Chain.weights`) needs algebraic trait implementations. Crucially, it must support types that do not implement `Zero` (like `CausalMultiVector`) to enable seamless composition.

**Implementation**:

> [!NOTE]
> This should be done in `deep_causality_sparse` crate.

1. **Add `algebra` module** to `deep_causality_sparse`:
   - `deep_causality_sparse/src/types/sparse_matrix/algebra/`
   - Implement `AbelianGroup`, `Module<S>` for `CsrMatrix<T>`

2. **Implement Contextual Sparsity**:
   - Add methods to `CsrMatrix` that accept an explicit `zero` value.
   - This allows `Chain<CausalMultiVector>` to work by passing a zero multivector (constructed with metric).

```rust
impl<T> CsrMatrix<T>
where
    T: Copy + PartialEq, // Removed Zero bound
{
    /// Creates a matrix from triplets with an explicit zero value.
    pub fn from_triplets_with_zero(
        rows: usize, 
        cols: usize, 
        triplets: &[(usize, usize, T)], 
        zero: T
    ) -> Result<Self, SparseMatrixError> {
        // ... implementation using `zero` instead of `T::zero()` ...
    }

    /// Adds two matrices with an explicit zero value for sparsity checks.
    pub fn add_with_zero(&self, rhs: &Self, zero: T) -> Self {
        // ... implementation checking `val == zero` ...
    }
}
```

3. **Example** (`group.rs`):
```rust
impl<T> CsrMatrix<T>
where
    T: AbelianGroup + Copy,
{
    pub fn zero(rows: usize, cols: usize) -> Self {
        Self { rows, cols, row_ptrs: vec![0; rows + 1], col_indices: vec![], values: vec![] }
    }

    pub fn add(&self, rhs: &Self) -> Self {
        // Standard addition assuming T: Zero if needed, or using algebraic properties
        // ...
    }
}
```

### Priority 4: Documentation and Examples

**Rationale**: Demonstrate the power of algebraic composition.

**Implementation**:

1. **Example**: `deep_causality_topology/examples/chain_algebra.rs`
   - Create a simplicial complex
   - Define chains with `Complex<f64>` coefficients
   - Compute boundary: `∂(c)`
   - Verify: `∂∂ = 0` (fundamental property)

2. **Example**: `deep_causality_topology/examples/differential_field.rs`
   - Create a triangulated manifold
   - Define a scalar field `f: M → ℝ`
   - Compute Laplacian `Δf`
   - Demonstrate heat equation: `∂f/∂t = Δf`

3. **Example**: `deep_causality_topology/examples/hodge_theory.rs`
   - Demonstrate Hodge decomposition
   - Use `Field` trait for harmonic forms

## Challenges & Solutions

### The "Zero" Trait Conflict

**Problem**: 
- `CsrMatrix` typically requires `T: Zero` to identify sparse elements.
- `CausalMultiVector` **cannot** implement `Zero` because constructing a zero vector requires a `Metric` (runtime context).
- Therefore, `Chain<CausalMultiVector>` is impossible with standard `CsrMatrix` traits.

**Solution: Contextual Sparsity**
1. **Enhance `CsrMatrix`**: Add `*_with_zero` methods (e.g., `add_with_zero`, `from_triplets_with_zero`) that take an explicit zero value.
2. **Update `Chain`**: 
   - For `T: Zero`, use standard methods.
   - For `T` without `Zero`, `Chain` must be constructed with a "zero context" or the user must manage the sparsity manually.
   - *Recommendation*: `Chain` methods should accept an optional context or zero value for advanced types.

## Proposed Changes

### Directory Structure

```
deep_causality_topology/src/types/
├── chain/
│   ├── algebra/
│   │   ├── mod.rs        [NEW]
│   │   ├── group.rs      [NEW] - zero, add, sub, neg
│   │   ├── module.rs     [NEW] - scale
│   │   └── traits.rs     [NEW] - AbelianGroup, Module
│   ├── arithmetic/
│   │   └── mod.rs        [NEW] - std::ops implementations
│   └── mod.rs            [MODIFY] - re-export algebra
├── manifold/
│   ├── differential/
│   │   ├── mod.rs        [NEW]
│   │   ├── exterior.rs   [NEW] - Exterior derivative
│   │   ├── hodge.rs      [NEW] - Hodge star operator
│   │   └── laplacian.rs  [NEW] - Laplace-Beltrami operator
│   └── mod.rs            [MODIFY] - add Field-bounded methods
└── ...
```

### deep_causality_sparse Updates

```
deep_causality_sparse/src/types/sparse_matrix/
├── algebra/
│   ├── mod.rs       [NEW]
│   ├── group.rs     [NEW] - zero, add, sub, neg
│   ├── module.rs    [NEW] - scale
│   └── traits.rs    [NEW] - AbelianGroup, Module
└── mod.rs           [MODIFY] - Add `*_with_zero` methods
```

## Implementation Phases

### Phase 1: Chain Algebraic Structure (Week 1-2)
- [x] Implement `Chain::zero`, `add`, `sub`, `neg` methods
- [x] Implement `Chain::scale` method
- [x] Implement standard operators (`Add`, `Sub`, `Neg`)
- [x] Implement `AbelianGroup`, `Module<S>` traits (Note: Cannot implement due to Zero trait constraint)
- [x] Add unit tests for chain algebra
- [ ] Verify `∂(c₁ + c₂) = ∂c₁ + ∂c₂` (boundary is homomorphism)

### Phase 2: CsrMatrix Algebraic Support (Week 2-3)
- [x] Implement `CsrMatrix<T>` algebraic methods in `deep_causality_sparse`
- [x] **Implement `from_triplets_with_zero` and `add_with_zero` in `CsrMatrix`**
- [x] Implement `AbelianGroup` for `CsrMatrix<T>`
- [x] Implement `Module<S>` for `CsrMatrix<T>`
- [x] Add unit tests for sparse matrix algebra
- [ ] Update `Chain` to use new `CsrMatrix` algebra

### Phase 3: Differential Operators on Manifolds (Week 3-4)
- [ ] Implement `Manifold::exterior_derivative` for `T: Field`
- [ ] Implement `Manifold::hodge_star` (requires metric)
- [ ] Implement `Manifold::laplacian`
- [ ] Add unit tests for differential operators
- [ ] Verify de Rham cohomology properties

### Phase 4: Examples and Documentation (Week 4-5)
- [ ] Create `chain_algebra.rs` example
- [ ] Create `differential_field.rs` example
- [ ] Create `hodge_theory.rs` example
- [ ] Update `README.md` with algebraic topology section
- [ ] Add mathematical background documentation

### Phase 5: Advanced Applications (Week 5-6)
- [ ] Implement cup product for cohomology (`H^k × H^l → H^{k+l}`)
- [ ] Implement Poincaré duality
- [ ] Demonstrate heat equation on manifolds
- [ ] Demonstrate wave equation using `Field` trait

## Verification Plan

### Automated Tests

1. **Chain Algebra Tests** (`deep_causality_topology/tests/chain_algebra_tests.rs`):
   - Test chain addition, subtraction, negation
   - Test scalar multiplication
   - Test boundary homomorphism: `∂(c₁ + c₂) = ∂c₁ + ∂c₂`
   - Test `∂∂ = 0` (consistency)
   - Test with `Complex<f64>`, `Quaternion<f64>` coefficients

2. **Differential Operator Tests** (`deep_causality_topology/tests/differential_tests.rs`):
   - Test `d² = 0` (exterior derivative is nilpotent)
   - Test Laplacian on simple manifolds (sphere, torus)
   - Compare numerical Laplacian with analytical solutions

3. **Composition Tests** (`deep_causality_topology/tests/composition_tests.rs`):
   - Test `Chain<CausalTensor<f64>>`
   - **Test `Chain<CausalMultiVector<f64>>` (using contextual sparsity)**
   - Test `Manifold` with `CausalMultiVector` fields

### Manual Verification

1. Run examples and verify output matches expected results
2. Check compilation with different type parameters (`f32`, `f64`, `Complex<f64>`)
3. Performance benchmarks for sparse matrix operations

## Dependencies

### New Dependencies
- None (all required traits exist in `deep_causality_num`)

### Updated Crates
- `deep_causality_topology` (primary implementation)
- `deep_causality_sparse` (algebraic support for `CsrMatrix` + **Contextual Sparsity**)

## Benefits

### 1. Algebraic Composition
```rust
// Chain with Complex coefficients (representing quantum amplitudes)
let chain: Chain<Complex<f64>> = /* ... */;
let superposition = chain1 + chain2;

// Chain with MultiVector coefficients (geometric algebra on chains)
// Requires explicit zero handling or context
let chain: Chain<CausalMultiVector<f64>> = /* ... */;
```

### 2. Differential Geometry
```rust
// Scalar field on manifold (heat distribution)
let mut temperature: Manifold<f64> = /* ... */;
let laplacian = temperature.laplacian(); // ∇²T

// Vector field using MultiVectors
let velocity_field: Manifold<CausalMultiVector<f64>> = /* ... */;
let curl = velocity_field.exterior_derivative(1); // d(velocity)
```

### 3. Physics Applications
- **Electromagnetism**: Maxwell's equations using differential forms
- **General Relativity**: Curvature tensors on manifolds
- **Quantum Field Theory**: Chains with operator-valued coefficients
- **Condensed Matter**: Topological invariants using chain complexes

## Risks and Mitigations

> [!WARNING]
> **Performance Risk**: Sparse matrix operations with generic types may be slower.
> **Mitigation**: Profile and optimize hot paths, use SIMD where possible.

> [!CAUTION]
> **API Complexity**: Adding too many bounds may confuse users.
> **Mitigation**: Provide clear examples and documentation for each trait requirement.

## Future Extensions

1. **Persistent Homology** with Field coefficients
2. **Spectral Sequences** using chain complexes
3. **Sheaf Cohomology** on simplicial complexes
4. **Quantum Topology** using non-commutative fields (Quaternions, Octonions)

## Summary

This plan implements algebraic traits on `Chain<T>` as the primary target, enabling:
- Homological algebra (chain complexes, boundary operators)
- Differential geometry (fields on manifolds with `Field` trait)
- Physics simulations (PDEs on triangulated domains)

The implementation follows the proven "indirect" pattern from `CausalTensor` and `CausalMultiVector`, ensuring type-safe composition while maintaining mathematical correctness. It also addresses the critical "Zero trait" limitation by introducing contextual sparsity in `CsrMatrix`.
