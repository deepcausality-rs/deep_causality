/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    PhysicsErrorEnum, StiffnessTensor, Strain, StressTensor, Temperature, hookes_law_kernel,
    thermal_expansion_kernel, von_mises_stress_kernel,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// hookes_law_kernel Tests
// =============================================================================

#[test]
fn test_hookes_law_kernel_valid() {
    // σ_ij = C_ijkl × ε_kl
    let mut stiffness_data = vec![0.0_f64; 81];
    stiffness_data[0] = 1.0;
    let stiffness =
        StiffnessTensor::<f64>::new(CausalTensor::new(stiffness_data, vec![3, 3, 3, 3]).unwrap());
    let strain = Strain::<f64>::new(CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap());

    let stress = hookes_law_kernel(&stiffness, &strain).unwrap();

    // sigma_ij = C_ijkl eps_kl. Only C_0000 is non-zero and every strain entry is 1, so
    // sigma_00 = 1 and every other component vanishes. The components are asserted, not only the
    // rank, which any rank-2 output would satisfy whatever the contraction did.
    assert_eq!(stress.inner().num_dim(), 2, "result is rank-2");
    let d: &[f64] = stress.inner().as_slice();
    assert_eq!(d.len(), 9, "3x3 stress tensor");
    assert!((d[0] - 1.0).abs() < 1e-12, "sigma_00 = {}", d[0]);
    for (i, v) in d.iter().enumerate().skip(1) {
        assert!(
            v.abs() < 1e-12,
            "sigma component {i} should vanish, got {v}"
        );
    }
}

#[test]
fn test_hookes_law_is_linear_in_the_strain() {
    // sigma(k eps) = k sigma(eps) for any stiffness tensor. No oracle needed.
    let mut stiffness_data = vec![0.0_f64; 81];
    stiffness_data[0] = 2.0;
    stiffness_data[40] = -1.5;
    let stiffness =
        StiffnessTensor::<f64>::new(CausalTensor::new(stiffness_data, vec![3, 3, 3, 3]).unwrap());
    let eps_base: Vec<f64> = (1..=9).map(|i| i as f64 * 0.25).collect();

    let base = hookes_law_kernel(
        &stiffness,
        &Strain::<f64>::new(CausalTensor::new(eps_base.clone(), vec![3, 3]).unwrap()),
    )
    .unwrap();

    for k in [0.5_f64, 2.0, -3.0] {
        let scaled_eps: Vec<f64> = eps_base.iter().map(|x| x * k).collect();
        let scaled = hookes_law_kernel(
            &stiffness,
            &Strain::<f64>::new(CausalTensor::new(scaled_eps, vec![3, 3]).unwrap()),
        )
        .unwrap();
        let a: &[f64] = base.inner().as_slice();
        let b: &[f64] = scaled.inner().as_slice();
        for (i, (x, y)) in a.iter().zip(b).enumerate() {
            assert!((y - k * x).abs() < 1e-12, "k = {k}, component {i}");
        }
    }
}

#[test]
fn test_hookes_law_kernel_dimension_mismatch_stiffness() {
    // Wrong rank stiffness (rank 2 instead of 4)
    let stiffness =
        StiffnessTensor::<f64>::new(CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap());
    let strain = Strain::<f64>::new(CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap());

    let result = hookes_law_kernel(&stiffness, &strain);

    match &result.unwrap_err().0 {
        PhysicsErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("Rank 4"));
        }
        _ => panic!("Expected DimensionMismatch error"),
    }
}

#[test]
fn test_hookes_law_kernel_dimension_mismatch_strain() {
    let stiffness =
        StiffnessTensor::<f64>::new(CausalTensor::new(vec![0.0; 81], vec![3, 3, 3, 3]).unwrap());
    let strain = Strain::<f64>::new(CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap());

    let result = hookes_law_kernel(&stiffness, &strain);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

// =============================================================================
// von_mises_stress_kernel Tests
// =============================================================================

#[test]
fn test_von_mises_stress_kernel_valid() {
    let mut stress_data = vec![0.0_f64; 9];
    stress_data[0] = 100e6;
    let stress = StressTensor::<f64>::new(CausalTensor::new(stress_data, vec![3, 3]).unwrap());

    let result = von_mises_stress_kernel(&stress);
    assert!(result.is_ok());

    let vm_stress = result.unwrap();
    assert!(vm_stress.value() > 0.0);
}

#[test]
fn test_von_mises_stress_kernel_hydrostatic() {
    let p = 50e6_f64;
    let stress = StressTensor::<f64>::new(
        CausalTensor::new(vec![p, 0.0, 0.0, 0.0, p, 0.0, 0.0, 0.0, p], vec![3, 3]).unwrap(),
    );

    let result = von_mises_stress_kernel(&stress);
    assert!(result.is_ok());

    let vm_stress = result.unwrap();
    assert!(
        vm_stress.value().abs() < 1e-6,
        "Hydrostatic stress should give near-zero Von Mises stress, got {}",
        vm_stress.value()
    );
}

#[test]
fn test_von_mises_stress_kernel_dimension_error() {
    let stress = StressTensor::<f64>::new(CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap());

    let result = von_mises_stress_kernel(&stress);

    match &result.unwrap_err().0 {
        PhysicsErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("3x3"));
        }
        _ => panic!("Expected DimensionMismatch error"),
    }
}

#[test]
fn test_von_mises_stress_kernel_rank_error() {
    let stress = StressTensor::<f64>::new(CausalTensor::new(vec![0.0; 27], vec![3, 3, 3]).unwrap());
    let result = von_mises_stress_kernel(&stress);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_von_mises_stress_kernel_shape_error() {
    let stress = StressTensor::<f64>::new(CausalTensor::new(vec![0.0; 12], vec![3, 4]).unwrap());
    let result = von_mises_stress_kernel(&stress);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

// =============================================================================
// thermal_expansion_kernel Tests
// =============================================================================

#[test]
fn test_thermal_expansion_kernel_valid() {
    let alpha: f64 = 12e-6;
    let delta_temp = Temperature::<f64>::new(100.0).unwrap();

    let result = thermal_expansion_kernel(alpha, delta_temp);
    assert!(result.is_ok());

    let strain = result.unwrap();
    assert_eq!(strain.shape(), vec![3, 3]);

    let expected = alpha * 100.0;
    assert!(
        (strain.data()[0] - expected).abs() < 1e-15,
        "Expected diagonal strain {}, got {}",
        expected,
        strain.data()[0]
    );
}

#[test]
fn test_thermal_expansion_kernel_zero_temp() {
    let alpha: f64 = 12e-6;
    let delta_temp = Temperature::<f64>::new(0.0).unwrap();

    let result = thermal_expansion_kernel(alpha, delta_temp);
    assert!(result.is_ok());

    let strain = result.unwrap();
    assert!(
        strain.data()[0].abs() < 1e-15,
        "Zero ΔT should give zero strain"
    );
}

// NOTE on defensively-unreachable extraction arms in `von_mises_stress_kernel`:
//   * mechanics.rs:70, 74 — the trace-extraction `else` arm. `EinSumOp::trace`
//     of a [3,3] tensor over axes (0,1) always produces a rank-0 / [1] scalar,
//     so `trace_tensor.shape()` is always scalar and the "Trace failed" arm at
//     line 74 never runs (line 70 is its non-taken predicate operand).
//   * mechanics.rs:95, 99-101 — the J2-extraction `else` arm. The double-axis
//     `EinSumOp::contraction` of the deviatoric stress with itself over (0,1),
//     (0,1) likewise yields a scalar, so "J2 calculation failed" is unreachable.
// Both guards require the einsum layer to return a non-scalar from a full
// contraction, which it never does for the fixed 3×3 inputs here.
