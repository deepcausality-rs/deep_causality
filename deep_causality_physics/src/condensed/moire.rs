/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Displacement, Energy, Momentum, PhysicsError, Ratio, Speed, Stiffness, TwistAngle};

use deep_causality_num::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

use crate::constants::GRAPHENE_LATTICE_CONST;
use deep_causality_topology::Manifold;
use std::f64::consts::PI;

/// Constructs the Bistritzer-MacDonald Continuum Hamiltonian for Twisted Bilayer Graphene (TBG).
///
/// This kernel implements the continuum model for TBG at small twist angles, constructing the
/// Moiré Hamiltonian in a plane-wave basis. It captures the interplay between the intralayer
/// Dirac cones and the spatially modulated interlayer tunneling potential.
///
/// # Physical Model
/// The Hamiltonian is constructed in the basis of Dirac spinors from the two layers, coupled by
/// interlayer tunneling matrices $T_j$.
///
/// $$ H = \begin{pmatrix} H_1(\mathbf{k}) & U(\mathbf{r}) \\ U^\dagger(\mathbf{r}) & H_2(\mathbf{k}) \end{pmatrix} $$
///
/// In momentum space, the tunneling potential $U(\mathbf{r})$ scatters states between Moiré
/// momentum shells.
///
/// # Implementation Details
/// *   **Basis Structure**: The Hamiltonian is returned as a block matrix.
///     *   Block (0,0): Layer 1 Dirac cone at the input $\mathbf{k}$.
///     *   Block (j,j): Layer 2 Dirac cones shifted by the Moiré reciprocal vectors $\mathbf{q}_j$.
///     *   Off-diagonal Blocks: Constant tunneling matrices $T_j$ connecting the shells.
/// *   **Cutoff**: Currently supports `shell_cutoff = 1`, which includes the central cone and
///     the first shell of 3 nearest-neighbor cones in the Moiré Brillouin Zone.
/// *   **Dimensions**: Returns an $8 \times 8$ complex tensor (matrix) representing the
///     coupled spinor states.
///
/// # Arguments
/// *   `twist_angle` - Moiré twist angle $\theta$ (in radians).
/// *   `interlayer_coupling` - Tunneling energy amplitude $w$ (in Joules or eV).
/// *   `fermi_velocity` - Monolayer graphene Fermi velocity $v_F$ (in m/s).
/// *   `k_point` - Momentum vector $\mathbf{k}$ in the Moiré Brillouin Zone (relative to the $\Gamma$ point).
/// *   `shell_cutoff` - Number of shells to include in the basis expansion (Must be 1).
///
/// # Returns
/// *   `Result<CausalTensor<Complex<f64>>, PhysicsError>` - The Hamiltonian matrix.
///
/// # Errors
/// *   `CalculationError` - If `shell_cutoff` is not 1.
pub fn bistritzer_macdonald_kernel(
    twist_angle: TwistAngle,
    interlayer_coupling: Energy,
    fermi_velocity: Speed,
    k_point: Momentum,
    shell_cutoff: usize,
) -> Result<CausalTensor<Complex<f64>>, PhysicsError> {
    if shell_cutoff != 1 {
        return Err(PhysicsError::CalculationError(
            "Only shell_cutoff=1 is currently supported".into(),
        ));
    }

    let theta = twist_angle.value();
    let w = interlayer_coupling.value();
    let vf = fermi_velocity.value();

    // Energy scale: hbar * vf
    // Assuming standard units (SI).
    let hbar = crate::constants::REDUCED_PLANCK_CONSTANT;
    let scale = hbar * vf;

    // Moiré momentum scale: k_theta = 8*pi / (3*a) * sin(theta/2)
    // This defines the size of the Moiré Brillouin Zone.
    let k_theta = (8.0 * PI / (3.0 * GRAPHENE_LATTICE_CONST)) * (theta / 2.0).sin();

    // Extract input momentum components (assuming Euclidean 3D metric layout: [scalar, x, y, z])
    let k_vec = k_point.inner().data();
    let kx = k_vec.get(1).copied().unwrap_or(0.0);
    let ky = k_vec.get(2).copied().unwrap_or(0.0);

    // Q vectors (Momentum shifts for Layer 2 shells)
    // q1 = (0, -k_theta)
    // q2 = (sqrt(3)/2 k_theta, 1/2 k_theta)
    // q3 = (-sqrt(3)/2 k_theta, 1/2 k_theta)
    let sqrt3 = 3.0f64.sqrt();
    let q_vectors = [
        (0.0, -k_theta),
        (sqrt3 * 0.5 * k_theta, 0.5 * k_theta),
        (-sqrt3 * 0.5 * k_theta, 0.5 * k_theta),
    ];

    // Helper to construct Tunnelling Matrices T_j
    // Form: T = w * [[1, z*], [z, 1]] where z = exp(i*phi)
    fn t_matrix(w: f64, phi: f64) -> [[Complex<f64>; 2]; 2] {
        let z = Complex::new(0.0, phi).exp(); // e^{i phi}
        let z_conj = Complex::new(z.re, -z.im);
        [
            [Complex::new(w, 0.0), Complex::new(w, 0.0) * z_conj],
            [Complex::new(w, 0.0) * z, Complex::new(w, 0.0)],
        ]
    }

    // Tunneling matrices with phase shifts 0, 2pi/3, -2pi/3
    let t1 = t_matrix(w, 0.0);
    let t2 = t_matrix(w, 2.0 * PI / 3.0);
    let t3 = t_matrix(w, -2.0 * PI / 3.0);
    let ts = [t1, t2, t3];

    // Dirac Hamiltonian helper: H(k) = hbar * vf * (sigma . k)
    // Uses standard Pauli matrices sigma_x and sigma_y.
    let dirac = |kx: f64, ky: f64| -> [[Complex<f64>; 2]; 2] {
        let k_plus = Complex::new(kx, ky);
        let k_minus = Complex::new(kx, -ky);
        [
            [Complex::new(0.0, 0.0), k_minus * scale],
            [k_plus * scale, Complex::new(0.0, 0.0)],
        ]
    };

    // Initialize 8x8 zero matrix
    // Layout: 4 blocks of 2x2 on diagonal (1 central + 3 shells)
    let mut data = vec![Complex::new(0.0, 0.0); 64];

    // Helper to set a 2x2 block in the 8x8 flattened matrix
    let set_block =
        |d: &mut Vec<Complex<f64>>, row_blk: usize, col_blk: usize, mat: [[Complex<f64>; 2]; 2]| {
            for (r, row) in mat.iter().enumerate() {
                for (c, &val) in row.iter().enumerate() {
                    let gr = row_blk * 2 + r;
                    let gc = col_blk * 2 + c;
                    d[gr * 8 + gc] = val;
                }
            }
        };

    // Block 0 (Central): Layer 1 Dirac cone at k
    set_block(&mut data, 0, 0, dirac(kx, ky));

    // Blocks 1..3 (Shells): Layer 2 Dirac cones at k + q_j
    // Also set off-diagonal tunneling blocks connecting Center <-> Shells
    for j in 0..3 {
        let blk_idx = j + 1;

        // Diagonal: H2(k + qj)
        let (qx, qy) = q_vectors[j];
        let kx2 = kx + qx;
        let ky2 = ky + qy;
        set_block(&mut data, blk_idx, blk_idx, dirac(kx2, ky2));

        // Off-diagonal: Tunneling
        // Block (blk_idx, 0): T_j (Layer 1 -> Layer 2)
        let t = ts[j];
        set_block(&mut data, blk_idx, 0, t);

        // Block (0, blk_idx): T_j^dagger (Layer 2 -> Layer 1)
        let t_dag = [
            [
                Complex::new(t[0][0].re, -t[0][0].im),
                Complex::new(t[1][0].re, -t[1][0].im),
            ],
            [
                Complex::new(t[0][1].re, -t[0][1].im),
                Complex::new(t[1][1].re, -t[1][1].im),
            ],
        ];
        set_block(&mut data, 0, blk_idx, t_dag);
    }

    CausalTensor::new(data, vec![8, 8]).map_err(PhysicsError::from)
}

/// Calculates the Föppl-von Kármán stress tensor (Simplified / Local).
///
/// This kernel implements the linear elastic constitutive relation for a 2D membrane
/// under plane stress conditions. It assumes the strain tensor $\epsilon$ is already known.
///
/// $$ \sigma_{ij} = \frac{E}{1-\nu^2} \left[ (1-\nu)\epsilon_{ij} + \nu \delta_{ij} \text{Tr}(\boldsymbol{\epsilon}) \right] $$
///
/// # Arguments
/// *   `displacement_u` - Input Strain Tensor $\boldsymbol{\epsilon}$ (Rank 2).
///     *Note*: Despite the name `displacement`, this argument expects the computed strain tensor in this simplified kernel.
/// *   `youngs_modulus` - Young's Modulus $E$ (Stiffness).
/// *   `poisson_ratio` - Poisson's Ratio $\nu$.
///
/// # Returns
/// *   `Result<CausalTensor<f64>, PhysicsError>` - Stress Tensor $\boldsymbol{\sigma}$.
pub fn foppl_von_karman_strain_simple_kernel(
    displacement_u: &Displacement,
    youngs_modulus: Stiffness,
    poisson_ratio: Ratio,
) -> Result<CausalTensor<f64>, PhysicsError> {
    let epsilon = displacement_u.inner();
    let e = youngs_modulus.value();
    let nu = poisson_ratio.value();

    if epsilon.num_dim() != 2 {
        return Err(PhysicsError::DimensionMismatch(
            "Strain tensor must be Rank 2".into(),
        ));
    }

    // Calculate Trace: Tr(epsilon) = eps_xx + eps_yy
    let trace_op = deep_causality_tensor::EinSumOp::<f64>::trace(epsilon.clone(), 0, 1);
    let trace_tensor = CausalTensor::ein_sum(&trace_op)?;
    let tr_eps: f64 = trace_tensor.data()[0];

    // Identity Tensor (delta_ij)
    let shape = epsilon.shape();
    let identity = CausalTensor::identity(shape)?;

    // Term 1: (1 - nu) * epsilon
    let term1 = epsilon.clone() * (1.0 - nu);

    // Term 2: nu * Tr(eps) * I
    let term2: CausalTensor<f64> = identity * (nu * tr_eps);

    // Sum terms
    let sum = term1 + term2;

    // Apply Prefactor: E / (1 - nu^2)
    let prefactor = e / (1.0 - nu * nu);
    let sigma = sum * prefactor;

    Ok(sigma)
}

/// Calculates the Föppl-von Kármán stress tensor using the complete theory (Field-based).
///
/// This kernel implements the large deflection plate theory, computing the strain field
/// from spatial derivatives of the in-plane ($\\mathbf{u}$) and out-of-plane ($w$) displacement fields.
///
/// # Physical Model
/// *   **Strain-Displacement Relation**: Includes the non-linear geometric term for large deflections.
///     $$ \epsilon_{ij} = \frac{1}{2}(\\partial_i u_j + \\partial_j u_i) + \frac{1}{2}\\partial_i w \\partial_j w $$
/// *   **Constitutive Relation**: Linear isotropic elasticity (Hooke's Law for plane stress).
///     $$ \sigma_{ij} = \frac{E}{1-\nu^2} ((1-\nu)\\epsilon_{ij} + \nu \delta_{ij} \text{Tr}(\\epsilon)) $$
///
/// # Numerical Method
/// *   **Gradient Calculation**: Uses the `exterior_derivative` operator from the provided `Manifold`
///     to approximate the fields $\\nabla \\mathbf{u}$ and $\\nabla w$.
/// *   **Approximations**:
///     *   The non-linear term $\\partial_i w \\partial_j w$ is approximated by the element-wise square
///         of the gradient field magnitudes if full tensor product fields are not supported by the Manifold.
///     *   The linear strain $\\epsilon_{linear}$ is approximated by the 1-form derivative $d\\mathbf{u}$.
///
/// # Arguments
/// *   `u_manifold` - Manifold containing the in-plane displacement field $\\mathbf{u}$ (as 0-forms).
/// *   `w_manifold` - Manifold containing the out-of-plane deflection field $w$ (as 0-forms).
/// *   `youngs_modulus` - Stiffness $E$.
/// *   `poisson_ratio` - Poisson's Ratio $\\nu$.
///
/// # Returns
/// *   `Result<CausalTensor<f64>, PhysicsError>` - The computed Stress Tensor field.
pub fn foppl_von_karman_strain_kernel(
    u_manifold: &Manifold<f64, f64>,
    w_manifold: &Manifold<f64, f64>,
    youngs_modulus: Stiffness,
    poisson_ratio: Ratio,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // 1. Non-linear term: 1/2 (grad w)^2
    // Calculate gradient dw (1-form) from w (0-form)
    let dw = w_manifold.exterior_derivative(0);

    // Approximate tensor product contraction as element-wise square for field magnitude
    let dw_sq = dw.clone() * dw.clone();
    let strain_nonlinear = dw_sq * 0.5;

    // 2. Linear term: Strain(u) approx du
    // Calculate gradient du from u
    let du = u_manifold.exterior_derivative(0);
    let strain_linear = du;

    // 3. Total Strain Field
    if strain_linear.shape() != strain_nonlinear.shape() {
        return Err(PhysicsError::DimensionMismatch(
            "Strain term shape mismatch between linear and non-linear components".into(),
        ));
    }
    let epsilon = strain_linear + strain_nonlinear;

    // 4. Stress Calculation
    // Apply constitutive relation to the strain field
    let e = youngs_modulus.value();
    let nu = poisson_ratio.value();
    let prefactor = e / (1.0 - nu * nu);

    // For this field-based approximation, we treat epsilon as the effective strain magnitude/trace.
    // Full tensor reconstruction would require a tensor-valued manifold.
    let sigma = epsilon * prefactor;

    Ok(sigma)
}
