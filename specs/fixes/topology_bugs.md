# Summary
- **Context**: The `extend` function in `TopologyWitness` implements the comonadic extension operation for the `Topology` type, which represents discrete fields defined on simplicial complexes.
- **Bug**: The `extend` function always creates a 1D tensor regardless of the original tensor's shape, causing shape information to be lost.
- **Actual vs. expected**: When extending a `Topology` containing a multi-dimensional tensor (e.g., shape `[2, 3]`), the result has a flattened 1D tensor (shape `[6]`) instead of preserving the original shape `[2, 3]`.
- **Impact**: This creates an inconsistency in the Functor/Comonad implementation where `fmap` correctly preserves tensor shapes but `extend` silently flattens them, leading to data corruption and unexpected behavior when chaining operations.

# Code with bug
```rust
fn extend<A, B, Func>(fa: &Topology<A>, mut f: Func) -> Topology<B>
where
    Func: FnMut(&Topology<A>) -> B,
    A: Zero + Copy + Clone,
    B: Zero + Copy + Clone,
{
    let size = fa.data.len();
    let mut result_vec = Vec::with_capacity(size);

    for i in 0..size {
        let mut view = fa.clone_shallow();
        view.cursor = i;

        let val = f(&view);
        result_vec.push(val);
    }

    Topology {
        complex: fa.complex.clone(),
        grade: fa.grade,
        // BUG 🔴: Always creates a 1D tensor with shape vec![size],
        // discarding the original multi-dimensional shape of fa.data
        data: CausalTensor::new(result_vec, vec![size]).unwrap(),
        cursor: 0,
    }
}
```

# Evidence

## Failing test

### Test script
```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{BoundedComonad, Functor};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{Topology, TopologyWitness};
use std::sync::Arc;

#[test]
fn test_topology_extend_preserves_shape() {
    let complex = Arc::new(create_triangle_complex());
    // Create a 2D tensor with shape [2, 3]
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let topology = Topology::new(complex, 0, data.clone(), 0);

    // Check original shape
    assert_eq!(data.shape(), &[2, 3]);

    // Extend: Just extract the value at cursor
    let extended = TopologyWitness::extend(&topology, |w| {
        TopologyWitness::extract(w)
    });

    // The extended topology should preserve the original shape [2, 3]
    // But currently it creates a 1D tensor with shape [6]
    assert_eq!(extended.data().shape(), data.shape(),
        "Shape should be preserved: expected {:?}, got {:?}",
        data.shape(), extended.data().shape());
}

#[test]
fn test_topology_fmap_preserves_shape() {
    let complex = Arc::new(create_triangle_complex());
    // Create a 2D tensor with shape [2, 3]
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let topology = Topology::new(complex, 0, data.clone(), 0);

    // Check original shape
    assert_eq!(data.shape(), &[2, 3]);

    // fmap: Just double each value
    let mapped = TopologyWitness::fmap(topology, |x| x * 2.0);

    // fmap should preserve the shape
    assert_eq!(mapped.data().shape(), data.shape(),
        "fmap should preserve shape: expected {:?}, got {:?}",
        data.shape(), mapped.data().shape());
}

#[test]
fn test_topology_fmap_then_extend_shape_inconsistency() {
    let complex = Arc::new(create_triangle_complex());
    // Start with a 2D tensor with shape [2, 3]
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let topology = Topology::new(complex, 0, data.clone(), 0);

    // Apply fmap - preserves shape
    let mapped = TopologyWitness::fmap(topology, |x| x * 2.0);
    assert_eq!(mapped.data().shape(), &[2, 3]);

    // Apply extend - flattens to 1D
    let extended = TopologyWitness::extend(&mapped, |w| {
        TopologyWitness::extract(w)
    });

    // This demonstrates the inconsistency:
    // fmap preserves [2,3] but extend flattens to [6]
    assert_ne!(extended.data().shape(), mapped.data().shape(),
        "Bug demonstration: extend flattens multi-dimensional tensors to 1D");
    assert_eq!(extended.data().shape(), &[6]);
}
```

### Test output
```
running 1 test
Original shape: [2, 3]
Extended shape: [6]
Extended data: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]

thread 'extensions::hkt_topology_tests::test_topology_extend_preserves_shape' (7273) panicked at deep_causality_topology/tests/extensions/hkt_topology_tests.rs:91:5:
assertion `left == right` failed: Shape should be preserved: expected [2, 3], got [6]
  left: [6]
 right: [2, 3]
test extensions::hkt_topology_tests::test_topology_extend_preserves_shape ... FAILED
```

The test demonstrates that:
1. `extend` flattens a 2D tensor with shape `[2, 3]` to a 1D tensor with shape `[6]`
2. `fmap` correctly preserves the shape `[2, 3]`
3. This inconsistency breaks the expected behavior when chaining functional operations

## Example

Consider a user working with vector fields on a simplicial complex, where each vertex has a 3D vector associated with it. They might represent this as a tensor with shape `[n_vertices, 3]`:

```rust
// 3 vertices, each with a 3D vector [x, y, z]
let data = CausalTensor::new(
    vec![1.0, 2.0, 3.0,  // vertex 0: [1, 2, 3]
         4.0, 5.0, 6.0,  // vertex 1: [4, 5, 6]
         7.0, 8.0, 9.0], // vertex 2: [7, 8, 9]
    vec![3, 3]
).unwrap();

let topology = Topology::new(complex, 0, data, 0);

// Apply fmap - shape preserved as [3, 3]
let scaled = TopologyWitness::fmap(topology, |v| v * 2.0);
assert_eq!(scaled.data().shape(), &[3, 3]); // ✓ Works correctly

// Apply extend - shape FLATTENED to [9]
let extended = TopologyWitness::extend(&scaled, |w| {
    TopologyWitness::extract(w)
});
assert_eq!(extended.data().shape(), &[9]); // ✗ Lost the [3, 3] structure!
```

After `extend`, the user loses the information that the data represents 3D vectors - it's now just a flat array of 9 values. Any subsequent operations expecting shape `[3, 3]` will fail or produce incorrect results.

## Inconsistency within the codebase

### Reference code
`deep_causality_topology/src/extensions/hkt_topology.rs:17-30` (fmap implementation)
```rust
impl Functor<TopologyWitness> for TopologyWitness {
    fn fmap<A, B, F>(fa: Topology<A>, f: F) -> Topology<B>
    where
        F: FnMut(A) -> B,
    {
        let new_data = CausalTensorWitness::fmap(fa.data, f);
        Topology {
            complex: fa.complex,
            grade: fa.grade,
            data: new_data,  // Preserves shape because CausalTensorWitness::fmap preserves it
            cursor: fa.cursor,
        }
    }
}
```

### Current code
`deep_causality_topology/src/extensions/hkt_topology.rs:46-80` (extend implementation)
```rust
fn extend<A, B, Func>(fa: &Topology<A>, mut f: Func) -> Topology<B>
where
    Func: FnMut(&Topology<A>) -> B,
    A: Zero + Copy + Clone,
    B: Zero + Copy + Clone,
{
    let size = fa.data.len();
    let mut result_vec = Vec::with_capacity(size);

    for i in 0..size {
        let mut view = fa.clone_shallow();
        view.cursor = i;
        let val = f(&view);
        result_vec.push(val);
    }

    Topology {
        complex: fa.complex.clone(),
        grade: fa.grade,
        data: CausalTensor::new(result_vec, vec![size]).unwrap(), // Always 1D!
        cursor: 0,
    }
}
```

### Contradiction

The `fmap` implementation delegates to `CausalTensorWitness::fmap`, which correctly preserves the tensor shape (it extracts the shape before mapping and recreates the tensor with the same shape). However, the `extend` implementation manually constructs a new `CausalTensor` with a hardcoded 1D shape `vec![size]`, ignoring the original shape.

This inconsistency violates the expected behavior of functorial operations, which should preserve the structure of the container. In category theory and functional programming, both `fmap` and `extend` should preserve the "shape" or "structure" of the functor/comonad - only the values should change, not the dimensionality or layout.

# Full context

The `Topology` type represents a discrete field defined on a simplicial complex (a mesh-like structure used in computational physics and geometry). The `data` field is a `CausalTensor<T>` that stores values associated with geometric elements (vertices, edges, faces, etc.). The `cursor` field is used for comonadic operations to focus on a specific element.

The `TopologyWitness` implements the `BoundedComonad` trait from the `deep_causality_haft` higher-kinded types library. This enables functional programming patterns like:
- `fmap`: Transform each value in the tensor
- `extract`: Get the value at the cursor position
- `extend`: Apply a context-dependent transformation (the function receives the whole `Topology` and can query neighbors, compute gradients, etc.)

These operations are used for implementing physics computations on discrete geometries, such as:
- Laplacian operators for diffusion equations
- Gradient computations for optimization
- Stencil operations for numerical methods

The bug affects any code that:
1. Creates a `Topology` with multi-dimensional tensor data
2. Uses `extend` to perform computations (very common for physics simulations)
3. Expects the shape to be preserved (which is the natural assumption)

The same bug exists in all related topology types:
- `deep_causality_topology/src/extensions/hkt_graph.rs:63`
- `deep_causality_topology/src/extensions/hkt_hypergraph.rs:63`
- `deep_causality_topology/src/extensions/hkt_manifold.rs:62`
- `deep_causality_topology/src/extensions/hkt_point_cloud.rs:60`

All of these files have the same pattern where `extend` creates a 1D tensor with `vec![size]`.

## External documentation

- [CausalTensor shape preservation in fmap](deep_causality_tensor/src/extensions/ext_hkt.rs:113-115)
```rust
fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
where
    Func: FnMut(A) -> B,
{
    let shape = m_a.shape().to_vec(); // Extract shape before moving data
    let new_data = m_a.data.into_iter().map(f).collect();
    CausalTensor::new(new_data, shape).expect("Shape should remain valid after fmap")
}
```

This shows that `CausalTensorWitness::fmap` explicitly preserves the shape by extracting it before transformation and passing it to the constructor. The `extend` implementations should follow the same pattern.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Limited real-world usage of multi-dimensional tensors**: All existing tests and examples use 1D tensors (shape `[n]`). The docstring for `Topology` says "CausalTensor is essentially a dense vector here", suggesting the original design intent was 1D-only. However, the type system allows multi-dimensional tensors, and there's no validation to enforce this constraint.

2. **No validation in constructor**: The `Topology::new` constructor accepts any `CausalTensor<T>` without checking if it's 1D. If the design intent is 1D-only, there should be a runtime check or the type should be `Vec<T>` instead of `CausalTensor<T>`.

3. **Shape preservation assumed for 1D**: When the tensor is 1D (shape `[n]`), creating a new tensor with `vec![size]` happens to produce the same shape, so the bug doesn't manifest. The issue only appears with multi-dimensional tensors.

4. **Inconsistency between operations**: Since `fmap` preserves shape but `extend` doesn't, users might successfully use `fmap` and only encounter problems when using `extend`, making it harder to identify the root cause.

5. **Spec matches implementation**: The specification in `specs/implemented/topoplogy.md:247` shows the same buggy code (`vec![size]`), suggesting this was the original design. However, the spec was likely copied from the implementation without critical review of the shape handling.

6. **Category theory abstraction leak**: Users familiar with comonads expect both `fmap` and `extend` to preserve structure. The inconsistency violates these expectations, but developers unfamiliar with category theory might not recognize this as a bug.

# Recommended fix

The `extend` function should preserve the original tensor shape by extracting it before the transformation loop and passing it to the `CausalTensor::new` constructor:

```rust
fn extend<A, B, Func>(fa: &Topology<A>, mut f: Func) -> Topology<B>
where
    Func: FnMut(&Topology<A>) -> B,
    A: Zero + Copy + Clone,
    B: Zero + Copy + Clone,
{
    let size = fa.data.len();
    let shape = fa.data.shape().to_vec(); // FIX 🟢: Extract the original shape
    let mut result_vec = Vec::with_capacity(size);

    for i in 0..size {
        let mut view = fa.clone_shallow();
        view.cursor = i;

        let val = f(&view);
        result_vec.push(val);
    }

    Topology {
        complex: fa.complex.clone(),
        grade: fa.grade,
        data: CausalTensor::new(result_vec, shape).unwrap(), // FIX 🟢: Use original shape
        cursor: 0,
    }
}
```

This matches the pattern used in `CausalTensorWitness::fmap` and ensures consistency across all functorial operations.

# Related bugs

The same bug exists in all other topology type implementations:
- `deep_causality_topology/src/extensions/hkt_graph.rs:45-72` - `GraphWitness::extend`
- `deep_causality_topology/src/extensions/hkt_hypergraph.rs:45-73` - `HypergraphWitness::extend`
- `deep_causality_topology/src/extensions/hkt_manifold.rs:44-71` - `ManifoldWitness::extend`
- `deep_causality_topology/src/extensions/hkt_point_cloud.rs:43-69` - `PointCloudWitness::extend`

All of these should be fixed with the same pattern: extract the original shape and pass it to `CausalTensor::new`.

# Summary
- **Context**: `Topology::new` is the primary constructor for the `Topology<T>` type, which represents a discrete field defined on a k-skeleton of a simplicial complex, used throughout the topology library for differential geometry operations.
- **Bug**: The constructor accepts invalid inputs without validation, allowing creation of malformed `Topology` instances with out-of-bounds cursors, invalid grades, or mismatched data/skeleton sizes.
- **Actual vs. expected**: The constructor currently accepts any values without checking invariants, whereas it should validate that cursor is within data bounds, grade exists in the complex, and data size matches the skeleton size.
- **Impact**: Invalid `Topology` instances cause panics at runtime in operations like `extract` (comonad), `cup_product`, and other topology operations, making debugging difficult since the error occurs far from the point where the invalid instance was created.

# Code with bug
`deep_causality_topology/src/types/topology/mod.rs`:
```rust
impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Self {
        Self {  // <-- BUG 🔴 No validation of inputs
            complex,
            grade,
            data,
            cursor,
        }
    }
}
```

# Evidence

## Failing test

### Test script
`deep_causality_topology/tests/types/topology/validation_bug_test.rs`:
```rust
/*
 * Test to demonstrate that Topology::new does not validate its inputs
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::Topology;
use std::sync::Arc;

#[test]
fn test_topology_allows_out_of_bounds_cursor() {
    let complex = Arc::new(create_triangle_complex());

    // The complex has 3 vertices (grade 0), so data has length 3
    // But we set cursor to 10, which is out of bounds
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let topology = Topology::new(complex.clone(), 0, data, 10);

    // This should have been rejected, but it succeeds
    assert_eq!(topology.cursor(), 10);
    assert_eq!(topology.data().as_slice().len(), 3);
    // cursor is 10 but data only has 3 elements - this is invalid!
}

#[test]
fn test_topology_allows_invalid_grade() {
    let complex = Arc::new(create_triangle_complex());

    // The complex only has dimensions 0, 1, 2 (max dimension is 2)
    // But we set grade to 5, which doesn't exist
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let topology = Topology::new(complex.clone(), 5, data, 0);

    // This should have been rejected, but it succeeds
    assert_eq!(topology.grade(), 5);
    assert_eq!(complex.max_simplex_dimension(), 2);
    // grade is 5 but max dimension is 2 - this is invalid!
}

#[test]
fn test_topology_allows_data_skeleton_mismatch() {
    let complex = Arc::new(create_triangle_complex());

    // Grade 0 (vertices) has 3 simplices, but we only provide 1 data value
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let topology = Topology::new(complex.clone(), 0, data, 0);

    // This should have been rejected, but it succeeds
    assert_eq!(topology.grade(), 0);
    assert_eq!(topology.data().as_slice().len(), 1);
    assert_eq!(complex.skeletons()[0].simplices().len(), 3);
    // data has 1 element but skeleton has 3 simplices - this is invalid!
}

#[test]
#[should_panic(expected = "Data/Skeleton mismatch")]
fn test_invalid_topology_causes_panic_in_cup_product() {
    let complex = Arc::new(create_triangle_complex());

    // Create topology with mismatched data - only 1 value for 3 vertices
    let data0 = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let topo0 = Topology::new(complex.clone(), 0, data0, 0);

    let data1 = CausalTensor::new(vec![0.5, 1.5, 2.5], vec![3]).unwrap();
    let topo1 = Topology::new(complex.clone(), 1, data1, 0);

    // This will panic because topo0 doesn't have enough data
    let _result = topo0.cup_product(&topo1);
}
```

### Test output
```
running 4 tests
test types::topology::validation_bug_test::test_topology_allows_data_skeleton_mismatch ... ok
test types::topology::validation_bug_test::test_invalid_topology_causes_panic_in_cup_product - should panic ... ok
test types::topology::validation_bug_test::test_topology_allows_invalid_grade ... ok
test types::topology::validation_bug_test::test_topology_allows_out_of_bounds_cursor ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 155 filtered out; finished in 0.00s
```

All four tests pass, confirming that:
1. Out-of-bounds cursor values are accepted (test 1)
2. Invalid grade values are accepted (test 2)
3. Data/skeleton size mismatches are accepted (test 3)
4. Invalid topologies cause panics in downstream operations (test 4)

## Example

Consider creating a `Topology` with an out-of-bounds cursor:

```rust
let complex = Arc::new(create_triangle_complex());
let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
let topology = Topology::new(complex, 0, data, 10);
```

This succeeds despite cursor=10 being invalid (data only has 3 elements). Later, when using the comonad's `extract` operation:

```rust
// From hkt_topology.rs:
fn extract<A>(fa: &Topology<A>) -> A
where
    A: Clone,
{
    fa.data
        .as_slice()
        .get(fa.cursor)  // Tries to access index 10
        .cloned()
        .expect("Cursor OOB")  // <-- Panics here!
}
```

The panic occurs in `extract`, far from where the invalid `Topology` was created, making debugging difficult. The error message "Cursor OOB" doesn't indicate what the cursor value was, what the valid range was, or where the invalid `Topology` was constructed.

## Inconsistency within the codebase

### Reference code
`deep_causality_topology/src/types/topology/cup_product.rs`:
```rust
pub fn cup_product(&self, other: &Topology<T>) -> Result<Topology<T>, TopologyError> {
    // ...validation checks...

    // Ensure both fields live on the same Complex
    if !Arc::ptr_eq(&self.complex, &other.complex) {
        return Err(TopologyError::GenericError("Complex Mismatch".to_string()));
    }

    // If grade exceeds manifold dimension, the result is zero.
    if r > self.complex.max_simplex_dimension() {
        // Returns a zero field...
    }

    // ...
}
```

### Current code
`deep_causality_topology/src/types/topology/mod.rs`:
```rust
impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Self {
        Self {
            complex,
            grade,
            data,
            cursor,
        }
    }
}
```

### Contradiction
The `cup_product` method performs careful validation and returns `Result<Topology<T>, TopologyError>` to handle invalid cases gracefully. However, the constructor performs no validation at all and returns a bare `Self`, allowing invalid instances to be created. This is inconsistent - the constructor is the natural place to enforce invariants that other methods rely on.

Furthermore, the codebase has a comprehensive `TopologyError` type with variants like `InvalidInput`, `IndexOutOfBounds`, and `InvalidGradeOperation` (see `src/errors/topology_error.rs`), but these are not used in the constructor.

# Full context

The `Topology<T>` struct represents a discrete field defined on the k-skeleton of a simplicial complex. It's a core type in the library, publicly exported in `src/lib.rs` and used throughout the codebase for differential geometry operations.

The struct has four fields that have mathematical constraints:
1. `complex`: The underlying simplicial complex (the "mesh")
2. `grade`: The dimension of simplices the data lives on (must be ≤ max dimension of complex)
3. `data`: A tensor containing values for each simplex at the given grade (length must match skeleton size)
4. `cursor`: The current focus point for comonadic extraction (must be < data.len())

The `Topology` type is used in several critical operations:
- **Comonadic operations** (`hkt_topology.rs`): The `extract` function reads the value at `cursor`, which panics if cursor is out of bounds with message "Cursor OOB"
- **Cup product** (`cup_product.rs`): Expects data length to match skeleton size at the given grade, panics with "Data/Skeleton mismatch" if invalid
- **Other geometric operations**: Various operations assume the topology is well-formed

The bug allows creating `Topology` instances that violate these invariants. When these invalid instances are used in downstream operations, they cause panics with error messages that don't help identify where the invalid instance was created.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Tests always use valid inputs**: All existing tests in the codebase create `Topology` instances with correct parameters that match the underlying complex structure. The test code follows good practices and doesn't accidentally create invalid topologies.

2. **Internal usage patterns are careful**: The library developers who write the test code understand the constraints and naturally create valid topologies. The constructor is primarily used in tests and examples, not in complex production scenarios.

3. **The bug manifests as panics, not silent corruption**: When invalid topologies are used, they cause panics rather than silent incorrect results. This means bugs are caught during development if they occur, but it also means the validation gap isn't discovered until someone tries to use the library incorrectly.

4. **Limited production usage**: The library appears to be relatively new (initial implementation in commit b80d08bb, with the topology type introduced recently). There may not yet be extensive production usage where users might accidentally create invalid topologies.

5. **Constructor simplicity masks the issue**: The constructor is so simple (just assigns fields) that it's not obvious validation is missing. It's easy to assume validation happens elsewhere or that callers are expected to validate inputs.

# Recommended fix

The constructor should validate its inputs and return a `Result<Self, TopologyError>` instead of `Self`. The validation should check:

1. **Cursor bounds**: `cursor < data.len()` - return `TopologyError::IndexOutOfBounds` if violated
2. **Grade validity**: `grade <= complex.max_simplex_dimension()` - return `TopologyError::InvalidGradeOperation` if violated
3. **Data/skeleton size match**: `data.len() == complex.skeletons()[grade].simplices().len()` - return `TopologyError::InvalidInput` if violated

Example fix:
```rust
impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {  // <-- FIX 🟢 Return Result
        // Validate grade
        if grade > complex.max_simplex_dimension() {
            return Err(TopologyError::InvalidGradeOperation(
                format!("grade {} exceeds max dimension {}",
                        grade, complex.max_simplex_dimension())
            ));
        }

        // Validate data size matches skeleton
        let expected_size = complex.skeletons()[grade].simplices().len();
        if data.len() != expected_size {
            return Err(TopologyError::InvalidInput(
                format!("data length {} does not match skeleton size {} for grade {}",
                        data.len(), expected_size, grade)
            ));
        }

        // Validate cursor bounds
        if cursor >= data.len() && data.len() > 0 {  // <-- FIX 🟢 Check cursor bounds
            return Err(TopologyError::IndexOutOfBounds(
                format!("cursor {} is out of bounds for data length {}",
                        cursor, data.len())
            ));
        }

        Ok(Self {
            complex,
            grade,
            data,
            cursor,
        })
    }
}
```

Note: This is a breaking API change. All call sites would need to be updated to handle the `Result`. Alternatively, for backwards compatibility, a new `try_new` method could be added while keeping the existing `new` method (though this delays fixing the root issue).
