/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    StiffnessTensor, Strain, StressTensor, Temperature, hookes_law, hookes_law_kernel,
    thermal_expansion, thermal_expansion_kernel, von_mises_stress, von_mises_stress_kernel,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// hookes_law Wrapper Tests
// =============================================================================

#[test]
fn test_hookes_law_wrapper_success() {
    // The previous fixture was an all-zero stiffness against an all-zero strain, so the answer
    // was zero for any kernel, and the assertion was `is_ok()` alone. Both are fixed: the
    // stiffness has a component, and the wrapper must carry the kernel's value rather than
    // merely succeed.
    let mut c = vec![0.0f64; 81];
    c[0] = 2.0; // C_0000
    let stiffness = StiffnessTensor::<f64>::new(CausalTensor::new(c, vec![3, 3, 3, 3]).unwrap());
    let strain = Strain::<f64>::new(CausalTensor::new(vec![1.5; 9], vec![3, 3]).unwrap());

    let effect = hookes_law(&stiffness, &strain);
    let carried = effect.value_cloned().unwrap();
    let direct = hookes_law_kernel(&stiffness, &strain).unwrap();
    assert_eq!(
        carried.inner().as_slice(),
        direct.inner().as_slice(),
        "hookes_law must carry the value its kernel produced"
    );
    // sigma_00 = C_0000 eps_00 = 2 * 1.5 = 3.
    let stress = effect.value_cloned().unwrap();
    let d: &[f64] = stress.inner().as_slice();
    assert!((d[0] - 3.0).abs() < 1e-12, "sigma_00 = {}", d[0]);
}

#[test]
fn test_hookes_law_wrapper_error() {
    let stiffness =
        StiffnessTensor::<f64>::new(CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap());
    let strain = Strain::<f64>::new(CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap());

    let effect = hookes_law(&stiffness, &strain);
    assert!(effect.is_err());
}

// =============================================================================
// von_mises_stress Wrapper Tests
// =============================================================================

#[test]
fn test_von_mises_stress_wrapper_success() {
    let stress = StressTensor::<f64>::new(
        CausalTensor::new(
            vec![100e6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            vec![3, 3],
        )
        .unwrap(),
    );

    // sigma = diag(100e6, 0, 0) is uniaxial, and the von Mises stress of a uniaxial state is the
    // axial stress itself: 100 MPa. `is_ok()` alone admitted any value.
    let effect = von_mises_stress(&stress);
    assert_eq!(
        effect.value_cloned().unwrap().value(),
        von_mises_stress_kernel(&stress).unwrap().value(),
        "von_mises_stress must carry the value its kernel produced"
    );
    let vm = effect.value_cloned().unwrap().value();
    assert!(
        (vm - 100.0e6).abs() / 100.0e6 < 1e-12,
        "uniaxial 100 MPa gives a von Mises stress of 100 MPa, got {vm}"
    );
}

#[test]
fn test_von_mises_stress_wrapper_error() {
    let stress = StressTensor::<f64>::new(CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap());

    let effect = von_mises_stress(&stress);
    assert!(effect.is_err());
}

// =============================================================================
// thermal_expansion Wrapper Tests
// =============================================================================

#[test]
fn test_thermal_expansion_wrapper_success() {
    let alpha = 12e-6;
    let delta_temp = Temperature::new(50.0).unwrap();

    // eps = alpha dT on the diagonal: 12e-6 * 50 = 6e-4.
    let effect = thermal_expansion(alpha, delta_temp);
    assert_eq!(
        effect.value_cloned().unwrap().as_slice(),
        thermal_expansion_kernel(alpha, delta_temp)
            .unwrap()
            .as_slice(),
        "thermal_expansion must carry the value its kernel produced"
    );
    let strain = effect.value_cloned().unwrap();
    let d: &[f64] = strain.as_slice();
    for (i, want) in [(0usize, 6.0e-4), (4, 6.0e-4), (8, 6.0e-4)] {
        assert!((d[i] - want).abs() < 1e-15, "eps[{i}] = {}", d[i]);
    }
    for i in [1usize, 2, 3, 5, 6, 7] {
        assert!(d[i].abs() < 1e-15, "off-diagonal eps[{i}] = {}", d[i]);
    }
}

// NOTE on materials/wrappers.rs:48 — the `Err(e)` arm of `thermal_expansion`.
// `thermal_expansion_kernel`'s only fallible step is
// `CausalTensor::<R>::identity(&[3, 3])`, which always succeeds for the fixed
// valid 3×3 shape, so the kernel always returns `Ok` and the wrapper's error
// arm can never run.
