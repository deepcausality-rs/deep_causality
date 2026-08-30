/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Algebra module for CausalMultiVector.

use crate::{CausalMultiVector, CausalMultiVectorError, Metric};
use alloc::vec;
use core::ops::{AddAssign, Neg, SubAssign};
use deep_causality_algebra::{AbelianGroup, Field, Module, NormedScalar, RealField, Ring};
use deep_causality_linear::{DenseMatrix, DenseVector, solve};

// Algebraic Composition
//
// 1.  **Complex Numbers (`Complex<f64>`):**
//     *   **Implements:** `Field` + `Copy` + `RealField` (if wrapped/adapted or treated as scalars).
//     *   **Path:** Uses **Tier 3**. `geometric_product` works correctly (assuming commutativity).
//     *   **Result:** Standard Quantum Mechanics (Spin(10)) works.
//
// 2.  **Quaternions (`Quaternion<f64>`):**
//     *   **Implements:** `Ring` + `Copy`.
//     *   **Does NOT Implement:** `Field` (Non-commutative).
//     *   **Path:** Uses **Tier 4**. `geometric_product_general` works correctly.
//     *   **Result:** Dixon Algebra nesting works. The non-commutative multiplication `q1 * q2` inside the geometric product loop is preserved.
//
// 3.  **Octonions (`Octonion<f64>`):**
//     *   **Implements:** `AbelianGroup` + `Copy`.
//     *   **Does NOT Implement:** `Ring` (non-associative; it is an `Algebra`).
//     *   **Path:** Uses **Tier 1**. `add`, `sub` work.
//     *   **Safety:** `geometric_product` is **Compile-Time Blocked**.
//
// You cannot accidentally multiply Octonion-MultiVectors (which would be undefined in standard Clifford terms).
// This is correct behavior.
//
// 4.  **Tensors (`CausalTensor<T>`):**
//     *   **Path:** `CausalMultiVector<f64>` implements `Ring` (via Tier 3/4).
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

// ============================================================================
// TIER 4: The Generalized Algebra (Non-Commutative Coefficients)
// Requirements: Ring (associative via MulMonoid; no commutativity guaranteed)
// Use Case: Dixon Algebra (Nesting), Tensor<MultiVector>
// ============================================================================

impl<T> CausalMultiVector<T>
where
    T: Ring + Copy,
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

// ============================================================================
// TIER 3: The Standard Clifford Algebra (Commutative Coefficients)
// Methods `normalize`, `commutator`, `inverse`, `geometric_product etc
// ============================================================================

// Internal implementation methods
impl<T> CausalMultiVector<T> {
    /// Computes the squared magnitude (squared norm) of the multivector.
    ///
    /// $$ ||A||^2 = \langle A \tilde{A} \rangle_0 $$
    pub(in crate::types::multivector) fn squared_magnitude_impl(&self) -> T
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        let reverse = self.reversion_impl();
        let product = self.geometric_product_impl(&reverse);
        product.data[0] // Scalar part
    }

    /// The multiplicative inverse $A^{-1}$, by solving a linear system in `deep_causality_linear`.
    ///
    /// # The previous formula was wrong
    ///
    /// It was $\tilde{A} / \langle A \tilde{A} \rangle_0$, quoted for versors. It does not hold
    /// even for those. For $A = 1 + 2e_1$ in $Cl(2)$: $A\tilde{A} = 5 + 4e_1$, so the formula
    /// returns $A/5$ and $A A^{-1} = 1 + 0.8 e_1$. The inverse is $(-1 + 2e_1)/3$. The reversion
    /// is the wrong involution and the scalar part of $A\tilde{A}$ is the wrong normaliser.
    ///
    /// Measured as $|A A^{-1} - 1|$ before this change: 0.8 for that versor, 0.93 for a general
    /// $Cl(2)$ element, 0.98 in $Cl(3)$, 0.99 in $Cl(4)$, 1.0 in $Cl(5)$.
    ///
    /// # What replaces it
    ///
    /// Left multiplication by `self` is a linear map on the $2^n$-dimensional coefficient space.
    /// Column `j` of its matrix is the coefficient vector of `self * e_j`, read straight off the
    /// geometric product, and $A^{-1}$ is the solution of $L_A x = 1$. Solving that is
    /// `deep_causality_linear`'s job.
    ///
    /// This needs no matrix *representation* of the algebra, which matters here: `to_matrix` is a
    /// faithful homomorphism only up to $n = 3$. Measured, $|\phi(AB) - \phi(A)\phi(B)|$ relative
    /// to scale is at machine epsilon for $n \le 3$ and of order one for $Cl(4)$, $Cl(5)$ and
    /// Minkowski, because those are quaternionic matrix algebras with no faithful real
    /// $4 \times 4$ form. A route through `to_matrix` would inherit that; this does not.
    ///
    /// Verified against $A A^{-1} = 1$ at $10^{-15}$ or better for $Cl(2)$ through $Cl(5)$ and
    /// Minkowski.
    ///
    /// # `NormedScalar`, not `Field`
    ///
    /// The LU factorisation pivots on magnitude. The alternative is `rref`, which needs only
    /// `Field` and picks the first non-zero pivot; measured on a $Cl(4)$ element with a $10^{-13}$
    /// leading coefficient it gave $3.2 \times 10^{-2}$ against LU's $1.3 \times 10^{-15}$.
    ///
    /// # Errors
    ///
    /// [`CausalMultiVectorError::zero_magnitude`] when `self` has no inverse. That now means the
    /// linear map is singular, which is the real condition — $1 + e_1$ is a null element with no
    /// inverse, and the old formula returned an answer for it.
    pub(in crate::types::multivector) fn inverse_impl(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign
            + NormedScalar,
    {
        let n = self.data.len();
        // Column j is the coefficients of `self * e_j`.
        let mut columns = vec![T::zero(); n * n];
        for j in 0..n {
            let mut basis = vec![T::zero(); n];
            basis[j] = T::one();
            let product = self.geometric_product_impl(&Self {
                data: basis,
                metric: self.metric,
            });
            for (i, coeff) in product.data.iter().enumerate() {
                columns[i * n + j] = *coeff;
            }
        }

        let map = DenseMatrix::from_vec(columns, n, n)
            .map_err(|_| CausalMultiVectorError::zero_magnitude())?;
        // The right-hand side is the identity multivector: scalar one, every other blade zero.
        let mut identity = vec![T::zero(); n];
        identity[0] = T::one();

        let solution = solve(&map, &DenseVector::from_vec(identity))
            .map_err(|_| CausalMultiVectorError::zero_magnitude())?;

        Ok(Self {
            data: solution.as_slice().to_vec(),
            metric: self.metric,
        })
    }

    /// Computes the dual of the multivector $A^*$ (CPU-only).
    pub(in crate::types::multivector) fn dual_impl(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign
            + NormedScalar,
    {
        let pseudo = Self::pseudoscalar(self.metric);
        let pseudo_inv = pseudo.inverse_impl()?;
        Ok(self.geometric_product_impl(&pseudo_inv))
    }
}

// Public API methods - Tier 3 operations (CPU version)
impl<T> CausalMultiVector<T>
where
    T: RealField + Copy,
{
    /// Normalizes the multivector to unit magnitude.
    pub fn normalize(&self) -> Self {
        let mag_sq = self.squared_magnitude_impl();
        if mag_sq <= T::epsilon() {
            return self.clone();
        }
        let scale_factor = T::one() / mag_sq.sqrt();
        self.scale(scale_factor)
    }
}

impl<T> CausalMultiVector<T>
where
    T: Field + Copy + RealField,
{
    /// Computes the Lie Commutator: $[A, B] = AB - BA$.
    /// Valid for all associative algebras.
    pub fn commutator(&self, rhs: &Self) -> Self {
        self.commutator_lie_impl(rhs)
    }

    /// The multiplicative inverse, so that `A * A.inverse()? == 1`.
    ///
    /// A wrapper over [`inverse_impl`](Self::inverse_impl), which is also what
    /// [`MultiVector::inverse`](crate::MultiVector::inverse) calls. One body, so the two cannot
    /// disagree — and they did: this method rejected any `|A|^2` at or below `T::epsilon()` while
    /// the trait rejected only exact zero. An inherent method wins method resolution, so
    /// `mv.inverse()` and `<_ as MultiVector<_>>::inverse(&mv)` answered differently for the same
    /// input, and the trait impl was unreachable without UFCS.
    ///
    /// The bound sits on the method rather than the block so [`commutator`](Self::commutator)
    /// keeps the looser one.
    pub fn inverse(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: NormedScalar,
    {
        self.inverse_impl()
    }

    /// The Geometric Product for Commutative Coefficients.
    /// This is the standard CPU implementation.
    pub fn geometric_product(&self, rhs: &Self) -> Self {
        self.geometric_product_impl(rhs)
    }

    /// Computes the Euclidean squared magnitude of a 3D spatial vector.
    ///
    /// For 4D Lorentzian multivectors with spatial components at indices 2, 3, 4
    /// (corresponding to x, y, z), this returns:
    ///
    /// $$ |v|^2_{\text{Euclidean}} = v_x^2 + v_y^2 + v_z^2 $$
    ///
    /// This differs from `squared_magnitude()` which applies the Lorentzian metric
    /// signature, potentially yielding negative values for spatial vectors.
    ///
    /// # Use Case
    /// Use this for classical EM quantities like energy density where the physical
    /// norm must be positive-definite.
    pub fn euclidean_squared_magnitude_3d(&self) -> T {
        let vx = self.data.get(2).copied().unwrap_or_else(T::zero);
        let vy = self.data.get(3).copied().unwrap_or_else(T::zero);
        let vz = self.data.get(4).copied().unwrap_or_else(T::zero);
        vx * vx + vy * vy + vz * vz
    }

    /// Computes the Euclidean magnitude of a 3D spatial vector.
    ///
    /// $$ |v|_{\text{Euclidean}} = \sqrt{v_x^2 + v_y^2 + v_z^2} $$
    pub fn euclidean_magnitude_3d(&self) -> T {
        self.euclidean_squared_magnitude_3d().sqrt()
    }

    /// Computes the 3D Euclidean cross product of two spatial vectors.
    ///
    /// For vectors with spatial components at indices 2, 3, 4 (x, y, z):
    ///
    /// $$ \mathbf{a} \times \mathbf{b} = (a_y b_z - a_z b_y, a_z b_x - a_x b_z, a_x b_y - a_y b_x) $$
    ///
    /// The result is returned in the same multivector format with the cross product
    /// components at indices 2, 3, 4.
    ///
    /// # Use Case
    /// Use this for classical EM quantities like the Poynting vector S = E × B.
    pub fn euclidean_cross_product_3d(&self, rhs: &Self) -> Self {
        let ax = self.data.get(2).copied().unwrap_or_else(T::zero);
        let ay = self.data.get(3).copied().unwrap_or_else(T::zero);
        let az = self.data.get(4).copied().unwrap_or_else(T::zero);

        let bx = rhs.data.get(2).copied().unwrap_or_else(T::zero);
        let by = rhs.data.get(3).copied().unwrap_or_else(T::zero);
        let bz = rhs.data.get(4).copied().unwrap_or_else(T::zero);

        // Cross product: c = a × b
        let cx = ay * bz - az * by;
        let cy = az * bx - ax * bz;
        let cz = ax * by - ay * bx;

        let mut result_data = vec![T::zero(); self.data.len()];
        result_data[2] = cx;
        result_data[3] = cy;
        result_data[4] = cz;

        Self {
            data: result_data,
            metric: self.metric,
        }
    }
}
