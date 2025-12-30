/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalMultiVector, CausalMultiVectorError, Metric};
use deep_causality_num::{AbelianGroup, AssociativeRing, Field, Module, RealField, Ring};

// Algebraic Composition
//
// 1.  **Complex Numbers (`Complex<f64>`):**
//     *   **Implements:** `Field` + `Copy` + `RealField` (if wrapped/adapted or treated as scalars).
//     *   **Path:** Uses **Tier 3**. `geometric_product` works correctly (assuming commutativity).
//     *   **Result:** Standard Quantum Mechanics (Spin(10)) works.
//
// 2.  **Quaternions (`Quaternion<f64>`):**
//     *   **Implements:** `AssociativeRing` + `Copy`.
//     *   **Does NOT Implement:** `Field` (Non-commutative).
//     *   **Path:** Uses **Tier 4**. `geometric_product_general` works correctly.
//     *   **Result:** Dixon Algebra nesting works. The non-commutative multiplication `q1 * q2` inside the geometric product loop is preserved.
//
// 3.  **Octonions (`Octonion<f64>`):**
//     *   **Implements:** `AbelianGroup` + `Copy`.
//     *   **Does NOT Implement:** `AssociativeRing` (Non-associative).
//     *   **Path:** Uses **Tier 1**. `add`, `sub` work.
//     *   **Safety:** `geometric_product` is **Compile-Time Blocked**.
//
// You cannot accidentally multiply Octonion-MultiVectors (which would be undefined in standard Clifford terms).
// This is correct behavior.
//
// 4.  **Tensors (`CausalTensor<T>`):**
//     *   **Path:** `CausalMultiVector<f64>` implements `AssociativeRing` (via Tier 3/4).
//     *   **Result:** `CausalTensor` accepts `CausalMultiVector`. You can do `tensor_a * tensor_b` where elements are MultiVectors.
//

// ============================================================================
// TIER 1: The Container (Storage & Linear Combinations)
// Requirements: AddGroup (Add, Sub, Neg, Zero)
// Use Case: Data storage, Accumulators, Octonion buffers (non-associative sums)
// ============================================================================

impl<T> CausalMultiVector<T>
where
    T: AbelianGroup + Copy,
{
    /// Creates a Zero vector (Additive Identity).
    pub fn zero(metric: Metric) -> Self {
        let size = 1 << metric.dimension();
        Self {
            data: vec![T::zero(); size],
            metric,
        }
    }

    /// Element-wise Addition.
    /// Checks metric compatibility.
    pub fn add(&self, rhs: &Self) -> Self {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch in add");
        let new_data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a + *b)
            .collect();

        Self {
            data: new_data,
            metric: self.metric,
        }
    }

    /// Element-wise Subtraction.
    pub fn sub(&self, rhs: &Self) -> Self {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch in sub");
        let new_data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a - *b)
            .collect();

        Self {
            data: new_data,
            metric: self.metric,
        }
    }
}

// ============================================================================
// TIER 2: The Vector Space (Scaling)
// Requirements: Module<S> (Vector Space over Scalar S)
// Use Case: Physics Vectors, Quantum States (Scaling Probability)
// ============================================================================

impl<T> CausalMultiVector<T> {
    /// Scales the multivector by a scalar value.
    /// $v' = s \cdot v$
    pub fn scale<S>(&self, scalar: S) -> Self
    where
        T: Module<S> + Copy, // T is the vector component
        S: Ring + Copy,      // S is the scalar (Must be Ring per Module trait)
    {
        // Note: Module definition usually implies Mul<S, Output=T>
        let new_data = self.data.iter().map(|v| *v * scalar).collect();
        Self {
            data: new_data,
            metric: self.metric,
        }
    }
}

// Extension: Normalization requires RealField logic (sqrt, epsilon)
impl<T> CausalMultiVector<T>
where
    T: RealField + Copy,
{
    /// Normalizes the multivector to unit magnitude.
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    pub fn normalize(&self) -> Self
    where
        T: Default + PartialOrd + Send + Sync + 'static,
    {
        // We assume squared_magnitude_impl returns T
        let mag_sq = self.squared_magnitude_impl();
        if mag_sq <= T::epsilon() {
            return self.clone();
        }
        let scale_factor = T::one() / mag_sq.sqrt();

        // Since T: RealField, it is a Ring, so it satisfies S: Ring for scale()
        self.scale(scale_factor)
    }

    /// Normalizes the multivector to unit magnitude.
    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    pub fn normalize(&self) -> Self {
        // We assume squared_magnitude_impl returns T
        let mag_sq = self.squared_magnitude_impl();
        if mag_sq <= T::epsilon() {
            return self.clone();
        }
        let scale_factor = T::one() / mag_sq.sqrt();

        // Since T: RealField, it is a Ring, so it satisfies S: Ring for scale()
        self.scale(scale_factor)
    }
}

// ============================================================================
// TIER 3: The Standard Clifford Algebra (Commutative Coefficients)
// Requirements: Field (Commutative Ring + Division)
// Use Case: Standard Model (Spin 10), Spacetime Algebra (Cl(1,3)), PGA
// ============================================================================

impl<T> CausalMultiVector<T>
where
    T: Field + Copy + RealField, // RealField implied for magnitude logic usually
{
    /// Computes the Lie Commutator: $[A, B] = AB - BA$.
    /// Valid for all associative algebras.
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    pub fn commutator(&self, rhs: &Self) -> Self
    where
        T: Default + PartialOrd + Send + Sync + 'static,
    {
        // Reuse the geometric product logic logic
        // Commutator = Geometric(A, B) - Geometric(B, A)
        // (Optimized impl handles this in one pass, see previous optimizations)
        self.commutator_lie_impl(rhs)
    }

    /// Computes the Lie Commutator: $[A, B] = AB - BA$.
    /// Valid for all associative algebras.
    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    pub fn commutator(&self, rhs: &Self) -> Self {
        self.commutator_lie_impl(rhs)
    }

    /// Computes the Multiplicative Inverse.
    /// $A^{-1} = \tilde{A} / |A|^2$ (For Versors).
    /// Requires Division (Field).
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    pub fn inverse(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Default + PartialOrd + Send + Sync + 'static,
    {
        let mag_sq = self.squared_magnitude_impl();

        // Check for zero divisor / null vector
        // Note: This is a simplified inverse logic valid for Versors.
        // General MV inversion is harder (requires matrix isomorphism).
        if mag_sq.abs() <= T::epsilon() {
            return Err(CausalMultiVectorError::zero_magnitude());
        }

        let conjugate = self.reversion_impl();
        let scale = T::one() / mag_sq;

        Ok(conjugate.scale(scale))
    }

    /// Computes the Multiplicative Inverse.
    /// $A^{-1} = \tilde{A} / |A|^2$ (For Versors).
    /// Requires Division (Field).
    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    pub fn inverse(&self) -> Result<Self, CausalMultiVectorError> {
        let mag_sq = self.squared_magnitude_impl();

        // Check for zero divisor / null vector
        // Note: This is a simplified inverse logic valid for Versors.
        // General MV inversion is harder (requires matrix isomorphism).
        if mag_sq.abs() <= T::epsilon() {
            return Err(CausalMultiVectorError::zero_magnitude());
        }

        let conjugate = self.reversion_impl();
        let scale = T::one() / mag_sq;

        Ok(conjugate.scale(scale))
    }

    /// The Geometric Product for Commutative Coefficients.
    ///
    /// With `mlx` feature on macOS aarch64: Automatically accelerates N≥6 algebras via GPU.
    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    pub fn geometric_product(&self, rhs: &Self) -> Self
    where
        T: Default + PartialOrd + Send + Sync + 'static,
    {
        self.geometric_product_impl(rhs)
    }

    /// The Geometric Product for Commutative Coefficients.
    /// This is the standard CPU implementation.
    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    pub fn geometric_product(&self, rhs: &Self) -> Self {
        self.geometric_product_impl(rhs)
    }
}

// ============================================================================
// TIER 4: The Generalized Algebra (Non-Commutative Coefficients)
// Requirements: AssociativeRing (No Commutativity guaranteed)
// Use Case: Dixon Algebra (Nesting), Tensor<MultiVector>
// ============================================================================

impl<T> CausalMultiVector<T>
where
    T: AssociativeRing + Copy,
{
    /// Generalized Geometric Product.
    ///
    /// Unlike the standard product, this does NOT assume coefficients commute.
    /// $ (a e_i) (b e_j) = (a b) (e_i e_j) $
    ///
    /// It strictly preserves the order `lhs * rhs` for coefficients.
    /// This allows `CausalMultiVector<Quaternion>` or `CausalMultiVector<Matrix>`.
    pub fn geometric_product_general(&self, rhs: &Self) -> Self {
        if self.metric != rhs.metric {
            panic!("Metric mismatch");
        }

        let dim = self.metric.dimension();
        let count = 1 << dim;
        let mut result_data = vec![T::zero(); count];

        // Dense Loop (Optimization: Add Sparsity check if T supports is_zero)
        for i in 0..count {
            // If T supports cheap zero check, add: if self.data[i].is_zero() continue;

            for j in 0..count {
                // 1. Compute Basis Sign/Index (The Geometry)
                // e_i * e_j = sign * e_k
                let (sign, k) = Self::basis_product(i, j, &self.metric);

                if sign == 0 {
                    continue;
                } // Degenerate metric

                // 2. Compute Coefficient Product (The Algebra)
                // CRITICAL: Order (i * j) must be preserved for non-commutative T.
                let term = self.data[i] * rhs.data[j];

                // 3. Accumulate
                // If sign is negative, we subtract.
                if sign > 0 {
                    result_data[k] = result_data[k] + term;
                } else {
                    result_data[k] = result_data[k] - term;
                }
            }
        }

        Self {
            data: result_data,
            metric: self.metric,
        }
    }
}
