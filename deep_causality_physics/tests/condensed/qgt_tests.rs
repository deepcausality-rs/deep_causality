/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    Energy, Length, QuantumEigenvector, QuantumMetric, QuantumVelocity,
    effective_band_drude_weight_kernel, quantum_geometric_tensor_kernel,
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
