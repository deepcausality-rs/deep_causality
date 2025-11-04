# Pre-Specification: Quaternion Implementation for `deep_causality_num`

## 1. Introduction

This document outlines the pre-specification for adding a `Quaternion` type to the `deep_causality_num` crate. Quaternions are a number system that extends complex numbers and are commonly used in 3D graphics and physics for representing rotations. The implementation will adhere to the existing conventions of the `deep_causality_num` crate, including generic float types and trait-based design.

## 2. Struct Definition

The `Quaternion` struct will be generic over a float type `F` that implements the `Float` trait. It will consist of a scalar part (`w`) and a vector part (`x`, `y`, `z`).

```rust
#[derive(Copy, Clone, PartialEq, Default)]
pub struct Quaternion<F>
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Rem<Output = Self>
        + Neg<Output = Self>
        + Sum
        + Product
        + PartialEq
        + Copy
        + Clone,
{
    pub w: F, // Scalar part
    pub x: F, // Vector part i
    pub y: F, // Vector part j
    pub z: F, // Vector part k
}
```

## 3. Core API and Traits

The `Quaternion` type will implement a set of core functionalities and traits to ensure it integrates seamlessly with the existing numerical ecosystem.

### 3.1 Constructors

*   `pub fn new(w: F, x: F, y: F, z: F) -> Self`: Creates a new quaternion from scalar and vector components.
*   `pub fn identity() -> Self`: Returns the identity quaternion (1 + 0i + 0j + 0k).
*   `pub fn from_axis_angle(axis: Vector3<F>, angle: F) -> Self`: Creates a quaternion from an axis-angle representation.
*   `pub fn from_euler_angles(roll: F, pitch: F, yaw: F) -> Self`: Creates a quaternion from Euler angles (roll, pitch, yaw).

### 3.2 Basic Arithmetic Operations

The `Quaternion` type will implement standard arithmetic traits from `std::ops`.

*   `Add<Output = Self>`: Quaternion addition.
*   `Sub<Output = Self>`: Quaternion subtraction.
*   `Mul<Output = Self>`: Quaternion multiplication. This will be implemented using Hamilton's rules, ensuring non-commutative multiplication as required for quaternion algebra.
*   `Mul<F, Output = Self>`: Scalar multiplication.
*   `Div<Output = Self>`: Quaternion division.
*   `Div<F, Output = Self>`: Scalar division.
*   `Neg<Output = Self>`: Unary negation.

### 3.3 Assignment Operations

*   `AddAssign<Output = Self>`: In-place quaternion addition.
*   `SubAssign<Output = Self>`: In-place quaternion subtraction.
*   `MulAssign<Output = Self>`: In-place quaternion multiplication.
*   `DivAssign<Output = Self>`: In-place quaternion division.

### 3.4 Unary Operations

*   `pub fn conjugate(&self) -> Self`: Returns the conjugate of the quaternion.
*   `pub fn norm_sqr(&self) -> F`: Computes the squared norm (magnitude squared) of the quaternion.
*   `pub fn norm(&self) -> F`: Computes the norm (magnitude) of the quaternion.
*   `pub fn normalize(&self) -> Self`: Returns a normalized quaternion (unit quaternion).
*   `pub fn inverse(&self) -> Self`: Returns the inverse of the quaternion.

### 3.5 Conversions and Other Utilities

*   `pub fn to_axis_angle(&self) -> (Vector3<F>, F)`: Converts the quaternion to an axis-angle representation.
*   `pub fn to_rotation_matrix(&self) -> Matrix3<F>`: Converts the quaternion to a 3x3 rotation matrix.
*   `pub fn dot(&self, other: &Self) -> F`: Computes the dot product with another quaternion.
*   `pub fn slerp(&self, other: &Self, t: F) -> Self`: Spherical linear interpolation.

### 3.6 Trait Implementations

*   `Debug`: For `{:?}` formatting.
*   `Display`: For `{}` formatting (e.g., `w + xi + yj + zk`).
*   `PartialEq`: For equality comparisons.
*   `Zero`: For `Quaternion::zero()` and `is_zero()`. 
*   `One`: For `Quaternion::one()` and `is_one()`. 
*   `Num`: The base numeric trait from `deep_causality_num`.
*   `Float`: Where applicable, implement methods from the `Float` trait (e.g., `is_nan`, `is_infinite`, `abs`, `exp`, `ln`, `powf`, `sqrt`). This will require careful consideration for complex-valued results.
*   `AsPrimitive<T>`: Conversion to primitive types (e.g., `f32`, `f64`) by taking the scalar part.
*   `FromPrimitive`: Conversion from primitive types to Quaternion (e.g., `from_f32` creates a real quaternion).
*   `NumCast`: For safe casting between numeric types.

## 4. Proposed Type Aliases

To support the `Quaternion` implementation, the following type aliases are proposed to be added to the `deep_causality_num` crate:

```rust
pub type Vector3<F> = [F; 3];
pub type Matrix3<F> = [[F; 3]; 3];
```

These aliases will provide convenient representations for 3D vectors and 3x3 matrices, which are essential for quaternion operations like axis-angle conversions and rotation matrix generation.

## 5. Placement of Type Aliases

The proposed `Vector3<F>` and `Matrix3<F>` type aliases will be placed in `src/alias/mod.rs` and re-exported from `lib.rs`.

## 6. Error Handling

For operations that can result in invalid or unrepresentable values, `NaN` will be used for `Float` errors (e.g., division by zero resulting in `NaN` components), and `Option<Quaternion>` will be returned for cases where a valid `Quaternion` cannot be constructed or computed (e.g., `from_axis_angle` with an invalid axis). This aligns with existing `deep_causality_num` conventions.

## 7. Integration with Complex<F>

The `Quaternion` struct is generic over `F: Float`, and `Complex<F>` also uses `F: Float`. This design inherently supports generic programming with `Float` types, allowing `Complex<F>` to be used where `Float` is expected, provided `Complex<F>` itself implements the `Float` trait. This enables a natural extension where quaternions can operate with complex numbers if needed.

## 8. Rotation Application (`v' = qvq^-1`)

While not a direct method on the `Quaternion` struct, the application of a quaternion to rotate a vector (`v' = qvq^-1`) is a fundamental use case. This operation would typically be implemented as a free function or an extension trait that takes a `Quaternion` and a `Vector3<F>` (or similar vector type) as input, returning a rotated `Vector3<F>`. This approach keeps the `Quaternion` struct focused on its mathematical properties while providing a clear way to perform rotations.

## 9. Performance Considerations

Optimizing for performance is crucial for a numerical library, especially for operations frequently used in simulations or real-time applications. The following strategies will be employed to ensure the `Quaternion` implementation is as efficient as possible:

*   **Inlining Small Functions**: Small, frequently called methods (e.g., component-wise arithmetic operations, getters for `w`, `x`, `y`, `z`) will be marked with `#[inline]` or `#[inline(always)]`. This encourages the Rust compiler to insert the function's code directly at the call site, eliminating function call overhead.

*   **Constant Time Complexity**: The majority of quaternion operations (addition, subtraction, multiplication, division, norm, inverse, conjugate) involve a fixed number of floating-point arithmetic operations. This inherently leads to constant time complexity (O(1)), regardless of the input values. For example:
    *   **Addition/Subtraction**: 4 additions/subtractions.
    *   **Multiplication (Hamilton Product)**: 16 multiplications and 12 additions/subtractions.
    *   **Norm Squared**: 4 multiplications and 3 additions.
    *   **Norm**: 4 multiplications, 3 additions, and 1 square root.
    *   **Inverse**: 4 multiplications, 3 additions, 1 square root, and 4 divisions.

*   **Leveraging `norm_sqr`**: Where only a comparison of magnitudes is needed (e.g., `min`, `max`, `clamp`), `norm_sqr()` will be used instead of `norm()` to avoid the computationally more expensive square root operation.

*   **Data Representation for SIMD**: While explicit SIMD (Single Instruction, Multiple Data) intrinsics are not a primary focus for the initial implementation due to Rust's evolving SIMD story, the `Quaternion` struct's `w, x, y, z` components are naturally aligned for potential future SIMD optimizations. Using an internal representation like `[F; 4]` could facilitate this, allowing compilers or future manual optimizations to process multiple components in parallel.

*   **Static Dispatch**: The use of generics (`Quaternion<F>`) and traits (`F: Float`) ensures static dispatch. This means that method calls are resolved at compile time, avoiding the overhead of dynamic dispatch (e.g., `dyn Trait` objects).

*   **Compiler Optimizations**: The implementation will rely on the Rust compiler's robust optimization passes (e.g., when compiling with `--release` and `-C opt-level=3`). Writing idiomatic Rust code that adheres to performance best practices will allow the compiler to generate highly optimized machine code.

*   **Floating Point Precision**: The generic `F` allows users to choose between `f32` and `f64`. `f32` operations are generally faster but offer less precision, while `f64` provides higher precision at the cost of potentially slower execution. The choice is left to the user based on their application's requirements.

## 10. Source Code Organization

Following the convention established by the `Complex` type, the `Quaternion` implementation will be organized into a dedicated module within `deep_causality_num/src/`.

*   **Module Structure**: A new folder `src/quaternion/` will be created, containing `mod.rs` and separate files for logical groupings of implementations, such as:
    *   `src/quaternion/arithmetic.rs` (for `Add`, `Sub`, `Mul`, `Div`, `Rem` traits)
    *   `src/quaternion/arithmetic_assign.rs` (for `AddAssign`, `SubAssign`, `MulAssign`, `DivAssign`, `RemAssign` traits)
    *   `src/quaternion/quaternion_number.rs` (for `QuaternionNumber` trait methods like `norm_sqr`, `norm`, `arg`, `conj`)
    *   `src/quaternion/as_primitive.rs` (for `AsPrimitive` trait)
    *   `src/quaternion/constructors.rs` (for `new`, `identity`, `from_axis_angle`, `from_euler_angles`)
    *   `src/quaternion/debug.rs` (for `Debug` trait)
    *   `src/quaternion/display.rs` (for `Display` trait)
    *   `src/quaternion/float.rs` (for `Float` trait implementation for `Quaternion<F>`)
    *   `src/quaternion/from_primitives.rs` (for `FromPrimitive` trait)
    *   `src/quaternion/identity.rs` (for `Zero`, `One`, `ConstZero`, `ConstOne` traits)
    *   `src/quaternion/neg.rs` (for `Neg` trait)
    *   `src/quaternion/num_cast.rs` (for `NumCast` trait)
    *   `src/quaternion/part_ord.rs` (for `PartialOrd` trait)
    *   `src/quaternion/to_primitive.rs` (for `ToPrimitive` trait)

*   **Module Export**: The main `src/quaternion/mod.rs` file will declare these sub-modules, and the `Quaternion` struct and its associated traits/functions will be re-exported from `deep_causality_num/src/lib.rs`.

## 11. Test Strategy

Testing for the `Quaternion` implementation will strictly follow the conventions established by the `Complex` type, ensuring comprehensive coverage and maintainability.

*   **Test Directory Structure**: A dedicated test folder `tests/quaternion/` will be created, mirroring the source code's internal module structure. For example:
    *   `tests/quaternion/arithmetic_tests.rs`
    *   `tests/quaternion/arithmetic_assign_tests.rs`
    *   `tests/quaternion/quaternion_number_tests.rs`
    *   ...and so on for each corresponding source file.

*   **Test File Naming**: Each test file will correspond to a source implementation file, with the `_tests.rs` suffix (e.g., `src/quaternion/arithmetic.rs` will have `tests/quaternion/arithmetic_tests.rs`).

*   **Test Utilities**: Shared test helper functions, such as approximate equality assertions for floating-point and quaternion types, will be placed in `deep_causality_num/src/utils_tests/utils_quaternion_tests.rs` (or similar, following the `utils_complex_tests` pattern) and imported into individual test files.

*   **Comprehensive Coverage**: Each method, trait implementation, and edge case (e.g., `NaN` inputs, zero divisors, identity values) will have dedicated unit tests to ensure correctness and adherence to mathematical properties.

## 12. Open Questions / Considerations
*   **Testing**: Comprehensive unit tests will be required for all methods and trait 

## implementation

During the implementation, the following amendments to the specs had to be made: 

*   **WHERE clause reduction**: The `WHERE` clause for `Quaternion` was reduced because the trait bounds of `PartialEq`, `Copy`, and `Clone` were conflicting with other trait bounds, thus they were removed from the explicit `Self` bounds.
*   **Float trait implementation**: The `Float` trait implementation for `Quaternion` was removed as it was deemed untenable to be implemented directly against the quaternion type.
*   **REM trait implementation**: The `Rem` trait implementation is not mathematically standard or feasible for quaternions.

