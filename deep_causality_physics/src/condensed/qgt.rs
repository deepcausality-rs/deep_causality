/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    BandDrudeWeight, Energy, Length, PhysicsError, QuantumEigenvector, QuantumMetric,
    QuantumVelocity,
};
use deep_causality_num::Complex;

/// Calculates the Quantum Geometric Tensor (QGT) component $Q_{ij}^n(\mathbf{k})$ for band $n$.
///
/// The QGT is a fundamental object in the geometry of quantum states, encapsulating both the
/// distance (metric) and curvature (Berry curvature) in the parameter space (Brillouin zone).
///
/// # Physical Model
/// Uses the sum-over-states (perturbative) formula derived from the Fubini-Study metric
/// (see, e.g., Kang et al., arXiv:2412.17809 for recent experimental applications):
///
/// $$ Q_{ij}^n = \sum_{m \neq n} \frac{\langle n | v_i | m \rangle \langle m | v_j | n \rangle}{(E_n - E_m)^2 + \epsilon} $$
///
/// where:
/// *   $|n\rangle, |m\rangle$: Eigenstates of the Hamiltonian.
/// *   $v_i = \partial_{k_i} H$: Velocity operator components.
/// *   $E_n, E_m$: Eigenenergies.
///
/// # Returns
/// A complex scalar $Q_{ij}$.
/// *   **Real Part**: Quantum Metric $g_{ij}$ (Symmetric).
/// *   **Imaginary Part**: $-\frac{1}{2} \Omega_{ij}$ (Berry Curvature, Antisymmetric).
///
/// # Arguments
/// *   `eigenvalues` - Energy eigenvalues $E_n$ (Rank 1 Tensor).
/// *   `eigenvectors` - Matrix of eigenstates $U$ (Rank 2 Tensor [basis, states]).
/// *   `velocity_i` - Velocity matrix $v_i$ components in the eigenbasis (Rank 2).
/// *   `velocity_j` - Velocity matrix $v_j$ components in the eigenbasis (Rank 2).
/// *   `band_n` - The target band index $n$.
/// *   `regularization` - Small $\epsilon$ parameter to regularize the denominator at band crossing points (degeneracies).
///
/// # Errors
/// *   `DimensionMismatch` - If input tensor shapes are inconsistent.
pub fn quantum_geometric_tensor_kernel(
    eigenvalues: &deep_causality_tensor::CausalTensor<f64>,
    eigenvectors: &QuantumEigenvector,
    velocity_i: &QuantumVelocity,
    velocity_j: &QuantumVelocity,
    band_n: usize,
    regularization: f64,
) -> Result<Complex<f64>, PhysicsError> {
    let ev_data = eigenvectors.inner().as_slice();
    let vi_data = velocity_i.inner().as_slice();
    let vj_data = velocity_j.inner().as_slice();
    let energies = eigenvalues.as_slice();

    let shape = eigenvectors.inner().shape();
    if shape.len() != 2 {
        return Err(PhysicsError::DimensionMismatch(
            "Eigenvectors must be Rank 2".into(),
        ));
    }
    let basis_size = shape[0];
    let num_states = shape[1];

    if band_n >= num_states {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Band index {} out of bounds (max {})",
            band_n,
            num_states - 1
        )));
    }

    if energies.len() != num_states {
        return Err(PhysicsError::DimensionMismatch(
            "Eigenvalues length mismatch".into(),
        ));
    }

    // Helper to compute matrix element <n | V | m>
    // Since we are given velocity vector columns V|m> directly (or pre-computed matrix elements?
    // The previous implementation assumed `velocity_i` stores the velocity vectors V|u_m>).
    // Specifically: dot product of <u_n| (conjugated) and V|u_m>.
    let inner_prod =
        |col_u: usize, col_v_matrix: &[Complex<f64>], col_v_idx: usize| -> Complex<f64> {
            let mut sum = Complex::new(0.0, 0.0);
            // Iterate over basis elements
            for b in 0..basis_size {
                let idx_u = b * num_states + col_u;
                let idx_v = b * num_states + col_v_idx;
                let u_val = ev_data[idx_u];
                let v_val = col_v_matrix[idx_v];
                // <u | v> = sum( u_k* * v_k )
                sum += Complex::new(u_val.re, -u_val.im) * v_val;
            }
            sum
        };

    let e_n = energies[band_n];
    let mut q_sum = Complex::new(0.0, 0.0);

    for (m, &e_m) in energies.iter().enumerate() {
        if m == band_n {
            continue;
        }

        let diff = e_n - e_m;
        // Regularized denominator: (E_n - E_m)^2 + epsilon
        let denom = diff * diff + regularization;

        // <n | v_i | m>
        let term1 = inner_prod(band_n, vi_data, m);
        // <m | v_j | n>
        let term2 = inner_prod(m, vj_data, band_n);

        let numerator = term1 * term2;
        q_sum += numerator / denom;
    }

    Ok(q_sum)
}

/// Calculates the Quasi-QGT $q_{ij}^n(\mathbf{k})$.
///
/// Wraps `quantum_geometric_tensor_kernel`. In the context of this library, the "Quasi-QGT"
/// refers to the same geometric object as the QGT, but its real and imaginary parts directly
/// correspond to experimentally accessible quantities (see Kang et al., arXiv:2412.17809).
/// This makes the Quasi-QGT a crucial bridge between theoretical QGT and experimental probes.
///
/// *   $\\text{Re}(q_{ij})$ maps to the band Drude weight (BDW).
/// *   $\\text{Im}(q_{ij})$ maps to the intrinsic orbital angular momentum (OAM).
pub fn quasi_qgt_kernel(
    eigenvalues: &deep_causality_tensor::CausalTensor<f64>,
    eigenvectors: &QuantumEigenvector,
    velocity_i: &QuantumVelocity,
    velocity_j: &QuantumVelocity,
    band_n: usize,
    regularization: f64,
) -> Result<Complex<f64>, PhysicsError> {
    quantum_geometric_tensor_kernel(
        eigenvalues,
        eigenvectors,
        velocity_i,
        velocity_j,
        band_n,
        regularization,
    )
}

/// Approximates the Effective Band Drude Weight ($D$) using band curvature and quantum geometry.
///
/// This kernel calculates the transport weight relevant for flat-band systems (like TBG)
/// where the geometric contribution is dominant.
///
/// $$ D \approx (D_{\text{conv}} + D_{\text{geom}}) \cdot a^2 $$
///
/// # Components
/// *   **Conventional ($D_{\text{conv}}$)**: $\partial^2 E / \partial \tilde{k}^2$. Proportional to the inverse effective mass.
///     Standard transport theory.
/// *   **Geometric ($D_{\text{geom}}$)**: $\tilde{g}_{ii} \cdot E_{\text{gap}}$. arises from the quantum metric of the wavefunction.
///     This "geometric lower bound" ensures transport even when the band is perfectly flat ($D_{\text{conv}} \approx 0$).
///
/// # Arguments
/// *   `energy_n` - Energy of the target band $E_n$.
/// *   `energy_0` - Reference energy (e.g., adjacent remote band) to define the gap $E_{\text{gap}} = |E_n - E_0|$.
/// *   `curvature_ii` - Dimensionless band curvature.
/// *   `quantum_metric` - Dimensionless Quantum Metric component $\tilde{g}_{ii}$.
/// *   `lattice_const` - Lattice constant $a$. Scales the result to physical units ($D \sim \text{Energy} \cdot \text{Length}^2$).
///     *   If inputs are already physical, pass $a=1.0$.
///
/// # Returns
/// *   `Result<BandDrudeWeight, PhysicsError>` - The physical Drude Weight.
pub fn effective_band_drude_weight_kernel(
    energy_n: Energy,
    energy_0: Energy,
    curvature_ii: f64,
    quantum_metric: QuantumMetric,
    lattice_const: Length,
) -> Result<BandDrudeWeight, PhysicsError> {
    if !curvature_ii.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Band curvature is not finite".into(),
        ));
    }

    let a = lattice_const.value();
    if a <= 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "Lattice constant must be positive, got {}",
            a
        )));
    }

    // Energy gap scale
    let gap = (energy_n.value() - energy_0.value()).abs();

    // Geometric contribution
    let geom_term = gap * quantum_metric.value();

    // Total dimensionless weight
    let dimensionless_weight = curvature_ii + geom_term;

    // Scale to physical units
    let scale_factor = a * a;
    let physical_weight = dimensionless_weight * scale_factor;

    if !physical_weight.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Resulting Drude Weight is not finite".into(),
        ));
    }

    BandDrudeWeight::new(physical_weight)
}
