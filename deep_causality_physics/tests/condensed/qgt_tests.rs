/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    Energy, Length, QuantumEigenvector, QuantumMetric, QuantumVelocity,
    effective_band_drude_weight_kernel, quantum_geometric_tensor_kernel, quasi_qgt_kernel,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_qgt_massive_dirac_k0() {
    // Massive Dirac Hamiltonian at k=0: H = m * sigma_z
    // Eigenstates: |0> (down, E=-m) = [0, 1]^T
    //              |1> (up, E=+m)   = [1, 0]^T
    // Velocities: vx = sigma_x, vy = sigma_y

    let m = 1.0;
    let energies = CausalTensor::new(vec![-m, m], vec![2]).unwrap();

    // Eigenvectors U = [[0, 1], [1, 0]]
    // Col 0: [0, 1], Col 1: [1, 0]
    // Layout [2, 2]: r0c0, r0c1, r1c0, r1c1
    // r0=[0, 1], r1=[1, 0]
    let u_data = vec![
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let u = QuantumEigenvector::new(CausalTensor::new(u_data, vec![2, 2]).unwrap());

    // Vx |0> = sigma_x [0, 1]^T = [1, 0]^T
    // Vx |1> = sigma_x [1, 0]^T = [0, 1]^T
    // Matrix cols: [[1, 0], [0, 1]]
    // Data: 1, 0, 0, 1
    let vx_data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
    ];
    let vx = QuantumVelocity::new(CausalTensor::new(vx_data, vec![2, 2]).unwrap());

    // Vy |0> = sigma_y [0, 1]^T = [-i, 0]^T
    // Vy |1> = sigma_y [1, 0]^T = [0, i]^T
    // Matrix cols: [[-i, 0], [0, i]]
    // Data: -i, 0, 0, i
    let vy_data = vec![
        Complex::new(0.0, -1.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 1.0),
    ];
    let vy = QuantumVelocity::new(CausalTensor::new(vy_data, vec![2, 2]).unwrap());

    // Calculate Q_xy for band 0
    let res = quantum_geometric_tensor_kernel(&energies, &u, &vx, &vy, 0, 1e-12);

    assert!(res.is_ok());
    let q = res.unwrap();

    // Expected: -0.25 i (Standard result for massive Dirac)
    // Formula check: ( <0|vx|1><1|vy|0> ) / (E0 - E1)^2
    // <0|vx|1> = [1, 0] . [0, 1] = 0? Wait.
    // |0> = [0, 1]. vx|1> = [0, 1]. <0|vx|1> = 1.
    // |1> = [1, 0]. vy|0> = [-i, 0]. <1|vy|0> = -i.
    // Num = 1 * -i = -i.
    // Denom = (-m - m)^2 = (-2)^2 = 4.
    // Result = -i / 4 = -0.25i. Correct.

    assert!((q.re - 0.0).abs() < 1e-10);
    assert!((q.im - (-0.25)).abs() < 1e-10);
}

#[test]
fn test_quasi_qgt_kernel_identical_to_qgt() {
    // quasi_qgt_kernel is a wrapper around quantum_geometric_tensor_kernel
    // Should return identical results
    let energies = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();
    let u_data = vec![Complex::new(1.0, 0.0); 4];
    let u = QuantumEigenvector::new(CausalTensor::new(u_data.clone(), vec![2, 2]).unwrap());
    let v_data = vec![Complex::new(0.1, 0.2); 4];
    let v = QuantumVelocity::new(CausalTensor::new(v_data.clone(), vec![2, 2]).unwrap());

    let qgt_result = quantum_geometric_tensor_kernel(&energies, &u, &v, &v, 0, 1e-12);
    let quasi_result = quasi_qgt_kernel(&energies, &u, &v, &v, 0, 1e-12);

    assert!(qgt_result.is_ok());
    assert!(quasi_result.is_ok());

    let qgt = qgt_result.unwrap();
    let quasi = quasi_result.unwrap();

    assert!((qgt.re - quasi.re).abs() < 1e-12);
    assert!((qgt.im - quasi.im).abs() < 1e-12);
}

#[test]
fn test_qgt_band_1() {
    // Test QGT for band 1 instead of band 0
    let energies = CausalTensor::new(vec![0.0, 2.0], vec![2]).unwrap();
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 4], vec![2, 2]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(vec![Complex::new(0.5, 0.0); 4], vec![2, 2]).unwrap(),
    );

    let res = quantum_geometric_tensor_kernel(&energies, &u, &v, &v, 1, 1e-12);
    assert!(res.is_ok());
}

#[test]
fn test_effective_band_drude_weight_kernel_physical() {
    // Case 1: Physical units input (a=1.0)
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(0.0).unwrap();
    let curvature = 0.5; // Physical units
    let metric = QuantumMetric::new(2.0).unwrap(); // Physical units
    let lattice = Length::new(1.0).unwrap(); // a=1

    let res = effective_band_drude_weight_kernel(energy_n, energy_0, curvature, metric, lattice);

    assert!(res.is_ok());
    let bdw = res.unwrap();

    // Gap = 1.0
    // Geom = 1.0 * 2.0 = 2.0
    // Total = (0.5 + 2.0) * 1^2 = 2.5
    assert!((bdw.value() - 2.5).abs() < 1e-10);
}

#[test]
fn test_effective_band_drude_weight_kernel_dimensionless() {
    // Case 2: Dimensionless input scaled by lattice constant
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(0.0).unwrap();
    let curvature = 0.5; // Dimensionless
    let metric = QuantumMetric::new(2.0).unwrap(); // Dimensionless
    let lattice = Length::new(2.0).unwrap(); // a=2

    let res = effective_band_drude_weight_kernel(energy_n, energy_0, curvature, metric, lattice);

    assert!(res.is_ok());
    let bdw = res.unwrap();

    // Gap = 1.0
    // Geom = 1.0 * 2.0 = 2.0
    // Dimensionless Sum = 2.5
    // Physical = 2.5 * a^2 = 2.5 * 4.0 = 10.0
    assert!((bdw.value() - 10.0).abs() < 1e-10);
}

// ============================================================================
// Error Path Tests
// ============================================================================

#[test]
fn test_qgt_error_eigenvector_not_rank2() {
    let energies = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();
    // Wrong shape: 1D instead of 2D
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 4], vec![4]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(vec![Complex::new(0.0, 0.0); 4], vec![2, 2]).unwrap(),
    );

    let res = quantum_geometric_tensor_kernel(&energies, &u, &v, &v, 0, 1e-12);
    assert!(res.is_err());
}

#[test]
fn test_qgt_error_band_index_out_of_bounds() {
    let energies = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 4], vec![2, 2]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(vec![Complex::new(0.0, 0.0); 4], vec![2, 2]).unwrap(),
    );

    // Band index 5 is out of bounds for 2 bands
    let res = quantum_geometric_tensor_kernel(&energies, &u, &v, &v, 5, 1e-12);
    assert!(res.is_err());
}

#[test]
fn test_qgt_error_eigenvalues_length_mismatch() {
    // 3 eigenvalues but 2 eigenvector columns
    let energies = CausalTensor::new(vec![0.0, 1.0, 2.0], vec![3]).unwrap();
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 4], vec![2, 2]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(vec![Complex::new(0.0, 0.0); 4], vec![2, 2]).unwrap(),
    );

    let res = quantum_geometric_tensor_kernel(&energies, &u, &v, &v, 0, 1e-12);
    assert!(res.is_err());
}

#[test]
fn test_effective_band_drude_weight_error_non_finite_curvature() {
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(0.0).unwrap();
    let curvature = f64::INFINITY; // Non-finite
    let metric = QuantumMetric::new(1.0).unwrap();
    let lattice = Length::new(1.0).unwrap();

    let res = effective_band_drude_weight_kernel(energy_n, energy_0, curvature, metric, lattice);
    assert!(res.is_err());
}

#[test]
fn test_effective_band_drude_weight_error_nan_curvature() {
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(0.0).unwrap();
    let curvature = f64::NAN; // Non-finite
    let metric = QuantumMetric::new(1.0).unwrap();
    let lattice = Length::new(1.0).unwrap();

    let res = effective_band_drude_weight_kernel(energy_n, energy_0, curvature, metric, lattice);
    assert!(res.is_err());
}

#[test]
fn test_effective_band_drude_weight_error_negative_lattice() {
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(0.0).unwrap();
    let curvature = 0.5;
    let metric = QuantumMetric::new(1.0).unwrap();
    // Length::new validates positive, so we need to use a workaround
    // Actually Length::new only validates >= 0, so 0 should fail in the kernel
    let lattice = Length::new(0.0).unwrap(); // Zero should fail

    let res = effective_band_drude_weight_kernel(energy_n, energy_0, curvature, metric, lattice);
    assert!(res.is_err());
}

#[test]
fn test_effective_band_drude_weight_zero_gap() {
    // Same energy → zero gap
    let energy_n = Energy::new(1.0).unwrap();
    let energy_0 = Energy::new(1.0).unwrap();
    let curvature = 0.5;
    let metric = QuantumMetric::new(2.0).unwrap();
    let lattice = Length::new(1.0).unwrap();

    let res = effective_band_drude_weight_kernel(energy_n, energy_0, curvature, metric, lattice);
    assert!(res.is_ok());

    // Gap = 0, Geom = 0, Total = 0.5 * 1 = 0.5
    let bdw = res.unwrap();
    assert!((bdw.value() - 0.5).abs() < 1e-10);
}
