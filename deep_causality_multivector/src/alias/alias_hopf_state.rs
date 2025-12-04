/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalMultiVector, CausalMultiVectorError, HilbertState, Metric, MultiVector};
use deep_causality_num::Complex;
use std::fmt::{Display, Formatter};

/// A point on the 3-Sphere ($S^3$), representing a unit spinor or rotor in 3D Euclidean space.
///
/// The Hopf Fibration maps this state to a point on the 2-Sphere ($S^2$) via the projection
/// $h(R) = R \sigma_3 \tilde{R}$.
///
/// This structure captures both the **Direction** (the point on $S^2$) and the **Phase/Twist**
/// (the position on the $S^1$ fiber).
///
/// # Applications
/// * **Quantum Mechanics:** Represents a Qubit state $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$. The projection is the Bloch Vector.
/// * **Robotics:** Represents a rotation without Gimbal Lock (Quaternion). The fiber is the "twist" redundancy.
/// * **Electromagnetism:** Represents Hopfion field configurations.
#[derive(Debug, Clone, PartialEq)]
pub struct HopfState(CausalMultiVector<f64>);

impl HopfState {
    /// Creates a new HopfState from raw coefficients.
    /// Enforces Euclidean(3) metric and Normalization.
    pub fn new(data: Vec<f64>) -> Result<Self, CausalMultiVectorError> {
        let metric = Metric::Euclidean(3);

        // Ensure we are in 3D space (8 components in Cl(3))
        if data.len() != 8 {
            return Err(CausalMultiVectorError::data_length_mismatch(8, data.len()));
        }

        let mv = CausalMultiVector::new(data, metric)?;

        // Enforce S^3 constraint: ||R|| = 1
        // (Project onto the hypersphere)
        let normalized = mv.normalize();

        Ok(Self(normalized))
    }

    /// Constructs a HopfState from two Complex numbers (Spinor formalism).
    ///
    /// Maps $(\alpha, \beta) \in \mathbb{C}^2$ to the 3-Sphere.
    /// $|\alpha|^2 + |\beta|^2 = 1$.
    ///
    /// This connects Standard QM notation to Geometric Algebra.
    pub fn from_spinor(alpha: Complex<f64>, beta: Complex<f64>) -> Self {
        // Mapping C^2 -> Cl(3) Even Subalgebra (Quaternions)
        // alpha = w + i z  -> Scalar + e12 (Bivector xy)
        // beta  = y + i x  -> e31 (Bivector zx) + e23 (Bivector yz)

        let mut data = vec![0.0; 8];

        // Scalar part (Real part of alpha)
        data[0] = alpha.re;

        // Bivector parts (Imaginary units)
        // We choose a mapping convention consistent with Pauli matrices:
        // I ~ -e123 (Pseudoscalar) or specific bivectors.
        // Standard Quaternion mapping:
        // 1 -> 1
        // i -> e23 (x-axis rotation)
        // j -> e31 (y-axis rotation)
        // k -> e12 (z-axis rotation)

        // Let's use the standard Rotor mapping:
        // Alpha (Identity/Z-phase):
        data[3] = alpha.im; // e12 (Generates Z rotation)
        // Beta (X/Y flips):
        data[5] = beta.im; // e13 ~ Y
        data[6] = beta.re; // e23 ~ X

        let mv = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
        Self(mv.normalize())
    }

    /// The Projection Map $h: S^3 \to S^2$.
    ///
    /// Returns the vector on the 2-Sphere (The "Shadow" or "Bloch Vector").
    /// $v = R \sigma_3 \tilde{R}$.
    pub fn project(&self) -> CausalMultiVector<f64> {
        // 1. Define the reference pole (North Pole / Z-axis / e3)
        let mut z_data = vec![0.0; 8];
        z_data[4] = 1.0; // Index 4 is e3 (binary 100)
        let sigma_3 = CausalMultiVector::new(z_data, Metric::Euclidean(3)).unwrap();

        // 2. Apply Sandwich Product: R * e3 * ~R
        let r = &self.0;
        let r_rev = r.reversion();

        let temp = r.geometric_product(&sigma_3);
        let vector = temp.geometric_product(&r_rev);

        // 3. Return only the Vector part (Grade 1) to ensure type purity
        vector.grade_projection(1)
    }

    /// Traverses the Fiber ($S^1$).
    ///
    /// Rotates the state by `phase` radians without changing the projection on $S^2$.
    /// This corresponds to the global phase $e^{i\theta}$ in QM or the "Twist" in robotics.
    ///
    /// $R' = R e^{-\frac{\theta}{2} \mathbf{I}}$
    /// (Where I is the generator of rotation around the pole, typically e12 for Z-axis).
    pub fn fiber_shift(&self, angle_rad: f64) -> Self {
        // Generator of the fiber (Z-rotation / Phase)
        let mut bivec_data = vec![0.0; 8];
        bivec_data[3] = 1.0; // e12
        let generator = CausalMultiVector::new(bivec_data, Metric::Euclidean(3)).unwrap();

        // Construct Rotor: exp(- B * angle/2)
        // Since B^2 = -1, exp is cos + sin
        let half_angle = angle_rad / 2.0;
        let cos = half_angle.cos();
        let sin = half_angle.sin();

        // R_phase = cos - sin * B
        let term_sin = generator * -sin;
        let term_cos = CausalMultiVector::scalar(cos, Metric::Euclidean(3));

        let phase_rotor = term_cos + term_sin;

        // Apply phase: R_new = R * R_phase
        let new_mv = self.0.geometric_product(&phase_rotor);

        Self(new_mv)
    }

    /// Access underlying algebra
    pub fn as_inner(&self) -> &CausalMultiVector<f64> {
        &self.0
    }
}

/// Conversion: Quantum State (Spinor) -> Topological Rotor (Hopf).
///
/// Maps a 2-level Quantum System (Qubit) into the geometry of the 3-Sphere.
/// $\psi = \alpha|0\rangle + \beta|1\rangle \to R \in S^3$.
///
/// This allows you to calculate the "Hopf Invariant" or "Berry Phase" of a quantum state.
impl TryFrom<&HilbertState> for HopfState {
    type Error = CausalMultiVectorError;

    fn try_from(quantum_state: &HilbertState) -> Result<Self, Self::Error> {
        let data = &quantum_state.as_inner().data;

        // 1. Extract Amplitudes (Alpha, Beta)
        // We assume the standard basis mapping:
        // Index 0 (Scalar) -> |0> (Alpha)
        // Index 1 (Vector) -> |1> (Beta) - Or wherever your basis states live
        //
        // Note: In Spin(10), the indices might be different.
        // For a standard Qubit simulation, we usually put Alpha at 0 and Beta at 1.
        if data.len() < 2 {
            return Err(CausalMultiVectorError::dimension_mismatch(2, data.len()));
        }

        let alpha = data[0];
        let beta = data[1]; // Assuming standard basis packing

        // 2. Map C^2 -> R^4 (Cl(3) Even Subalgebra)
        // We use the standard spinor-to-rotor isomorphism.
        // Alpha = w + I_z z
        // Beta  = y + I_z x  (Where I_z, I_y, I_x are bivectors)

        // Reusing the logic from 'from_spinor' we wrote earlier:
        Ok(HopfState::from_spinor(alpha, beta))
    }
}

/// Conversion: Topological Rotor (Hopf) -> Quantum State (Spinor).
///
/// Maps a geometric orientation back into Quantum Hilbert Space.
/// Useful for initializing a Qubit based on a geometric rotation.
impl From<HopfState> for HilbertState {
    fn from(hopf: HopfState) -> Self {
        // 1. Access the real Cl(3) coefficients
        let d = hopf.as_inner().data();

        // 2. Reconstruct Complex Amplitudes
        // Scalar + e12 (Z-rot) -> Alpha
        let re_alpha = d[0]; // Scalar
        let im_alpha = d[3]; // e12
        let alpha = Complex::new(re_alpha, im_alpha);

        // e23 (X) + e13 (Y) -> Beta
        // Note: Check signs based on your specific quaternion mapping preference
        let re_beta = d[6]; // e23
        let im_beta = d[5]; // e13
        let beta = Complex::new(re_beta, im_beta);

        // 3. Construct HilbertState
        // We need to respect the target metric (e.g. Spin(10) / Cl(0,10))
        let mut q_data = vec![Complex::new(0.0, 0.0); 1024]; // Standard size
        q_data[0] = alpha;
        q_data[1] = beta;

        HilbertState::new_spin10(q_data).unwrap()
    }
}

impl Display for HopfState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
