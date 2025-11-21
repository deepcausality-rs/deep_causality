/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{HilbertState, MultiVector};
use deep_causality_num::{Complex, Complex64, ComplexNumber, Zero};

/// Operations specific to Quantum Mechanics / Hilbert Spaces.
/// These correspond to Bra-Ket notation operations.
pub trait QuantumOps {
    /// The Hermitian Conjugate (The "Bra" <psi|)
    /// Corresponds to Reversion (Geometry) + Complex Conjugation (Coefficients).
    fn dag(&self) -> Self;

    /// The Inner Product <self | other>
    /// Returns the Probability Amplitude (Scalar).
    fn bracket(&self, other: &Self) -> Complex64;

    /// The Expectation Value <self | Operator | self>
    /// Returns the observable value (Scalar).
    fn expectation_value(&self, operator: &Self) -> Complex64;

    /// Normalizes the state so <psi|psi> = 1
    fn normalize(&self) -> Self;
}

impl QuantumOps for HilbertState {
    /// The Hermitian Conjugate: $\psi^\dagger$.
    ///
    /// In quantum mechanics, the Hermitian conjugate (or adjoint) of a ket $|\psi\rangle$ is the bra $\langle\psi|$.
    /// For operators, it generalizes to $O^\dagger$.
    /// In the context of Geometric Algebra with complex coefficients, this operation involves two steps:
    /// 1.  **Geometric Reversion**: Reversing the order of basis vectors within each blade. This changes the sign
    ///     of certain blades based on their grade (e.g., $(e_1 e_2)^\dagger = e_2 e_1 = -e_1 e_2$).
    /// 2.  **Complex Conjugation**: Taking the complex conjugate of all scalar coefficients.
    ///
    /// Mathematically, if $A = \sum_I (a_I + i b_I) e_I$, then $A^\dagger = \sum_I (a_I - i b_I) \tilde{e_I}$,
    /// where $\tilde{e_I}$ is the reversion of the basis blade $e_I$.
    ///
    /// # Rust Details
    /// This is implemented by first calling the `reversion()` method on the underlying `CausalMultiVector`
    /// to handle the geometric part, and then iterating through the resulting `data` vector to apply
    /// `c.conj()` (complex conjugate) to each `Complex64` coefficient.
    fn dag(&self) -> Self {
        // 1. Geometric Reversion
        let mut reversed = self.mv().reversion();

        // 2. Complex Conjugation of coefficients
        for c in reversed.data.iter_mut() {
            *c = c.conj();
        }

        Self::from(reversed)
    }

    /// The Inner Product: $\langle \psi | \phi \rangle$.
    ///
    /// # Physics
    /// In quantum mechanics, the inner product of two state vectors (a bra $\langle \psi |$ and a ket $| \phi \rangle$)
    /// results in a complex scalar, known as a probability amplitude. Its squared magnitude, $|\langle \psi | \phi \rangle|^2$,
    /// represents the probability of finding the system in state $|\psi\rangle$ given it was prepared in state $|\phi\rangle$.
    ///
    /// # Math
    /// The inner product is calculated as the scalar (grade 0) component of the geometric product of the
    /// Hermitian conjugate of the first state with the second state:
    /// $ \langle \psi | \phi \rangle = \text{ScalarPart}(\psi^\dagger \cdot \phi) $
    ///
    /// # Rust Details
    /// The `dag()` method is first called on `self` to obtain the bra $\langle \psi |$.
    /// Then, the `geometric_product()` of the resulting bra's underlying multivector and the `other` ket's
    /// underlying multivector (`other.mv()`) is computed. Finally, the scalar component (grade 0, index 0)
    /// is extracted from the result.
    fn bracket(&self, other: &Self) -> Complex64 {
        let bra = self.dag();

        // Geometric Product: Bra * Ket
        // The underlying CausalMultiVector handles metric consistency.
        let product = bra.mv().geometric_product(other.mv());

        // Extract Grade 0 (Scalar) component.
        // Returns Complex::zero() if for some reason the scalar component is not found,
        // though this should not happen in a valid CausalMultiVector.
        product.get(0).cloned().unwrap_or(Complex::zero())
    }

    /// The Expectation Value: $\langle \psi | \hat{O} | \psi \rangle$.
    ///
    /// # Physics
    /// The expectation value of an observable (represented by a Hermitian operator $\hat{O}$)
    /// for a quantum system in state $|\psi\rangle$ is the average value of many measurements
    /// of that observable. It is a real scalar.
    ///
    /// # Math
    /// The expectation value is calculated using the "sandwich" product:
    /// $ \langle \psi | \hat{O} | \psi \rangle = \text{ScalarPart}(\psi^\dagger \cdot \hat{O} \cdot \psi) $
    ///
    /// The operator `operator` is passed as `&Self`. In Furey's algebraic approach,
    /// operators (End(A)) and states (Ideals) can live in the same algebra (e.g., Cl(0,10)),
    /// so representing the operator as a `HilbertState` (or similar `Newtype`) is a valid type-safe approach.
    ///
    /// # Rust Details
    /// The implementation first obtains the bra $\langle \psi |$ using `self.dag()`.
    /// It then performs two sequential geometric products:
    /// 1.  `op.geometric_product(ket)`: Applies the operator $\hat{O}$ to the ket $|\psi\rangle$,
    ///     resulting in an intermediate state $|\phi\rangle = \hat{O}|\psi\rangle$.
    /// 2.  `bra.mv().geometric_product(&phi)`: Takes the inner product $\langle \psi | \phi \rangle$.
    /// 3.  Finally, the scalar (grade 0) component of the result is extracted.
    fn expectation_value(&self, operator: &Self) -> Complex64 {
        let bra = self.dag();
        let ket = self.mv();
        let op = operator.mv();

        // 1. Apply Operator: |phi> = \hat{O} |psi>
        let phi = op.geometric_product(ket);

        // 2. Closure: <psi | phi>
        let result = bra.mv().geometric_product(&phi);

        result.get(0).cloned().unwrap_or(Complex::zero())
    }

    /// Normalizes the state so $\langle \psi | \psi \rangle = 1$.
    ///
    /// # Physics
    /// In quantum mechanics, physical states are represented by normalized vectors in Hilbert space.
    /// The normalization condition $\langle \psi | \psi \rangle = 1$ ensures that the total probability
    /// of finding the system in any possible state is unity.
    ///
    /// # Math
    /// A state $|\psi\rangle$ is normalized by dividing it by its norm (magnitude):
    /// $ |\psi'\rangle = \frac{|\psi\rangle}{\sqrt{\langle \psi | \psi \rangle}} $
    /// where $\sqrt{\langle \psi | \psi \rangle}$ is the L2 norm of the state vector.
    /// The term $\langle \psi | \psi \rangle$ is the squared norm (or probability density), which is
    /// a real, non-negative scalar.
    ///
    /// # Rust Details
    /// 1.  The `bracket(self)` method is called to calculate $\langle \psi | \psi \rangle$.
    ///     Since this must be a real number, its real part (`.re`) is extracted.
    /// 2.  A check is performed for `norm_sq <= f64::EPSILON` to handle cases where the state is
    ///     effectively a zero vector, preventing division by zero or NaN results. In such cases,
    ///     the original (unnormalized) state is returned.
    /// 3.  The scaling factor is computed as `1.0 / norm_sq.sqrt()`.
    /// 4.  Each complex coefficient in the underlying `CausalMultiVector`'s `data` is multiplied
    ///     by this scaling factor.
    /// 5.  A new `HilbertState` is constructed using `Self::new_unchecked()`. This is safe because
    ///     normalization only scales coefficients and does not change the metric or the structure
    ///     of the multivector, thus preserving its validity with respect to its original construction.
    fn normalize(&self) -> Self {
        // 1. Calculate Norm Squared (Real number)
        let norm_sq = self.bracket(self).re; // .re gets the real part of Complex

        // Handle zero vectors to avoid NaN
        if norm_sq <= f64::EPSILON {
            return self.clone();
        }

        // 2. Calculate scaling factor
        let scale = 1.0 / norm_sq.sqrt();

        // 3. Scale all coefficients
        let new_data = self.mv().data.iter().map(|c| *c * scale).collect();

        // Reconstruct directly to avoid re-checking metric
        Self::new_unchecked(new_data, self.mv().metric)
    }
}
