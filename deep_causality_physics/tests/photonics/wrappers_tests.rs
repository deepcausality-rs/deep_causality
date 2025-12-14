/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    AbcdMatrix, ComplexBeamParameter, IndexOfRefraction, JonesVector, Length, RayAngle, RayHeight,
    Wavelength, beam_spot_size, lens_maker, ray_transfer, single_slit_irradiance, snells_law,
    stokes_from_jones,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_wrappers() {
    // Ray Transfer
    let m = AbcdMatrix::new(CausalTensor::identity(&[2, 2]).unwrap());
    let h = RayHeight::default();
    let a = RayAngle::default();
    assert!(ray_transfer(&m, h, a).is_ok());

    // Snells
    let n1 = IndexOfRefraction::new(1.0).unwrap();
    let n2 = IndexOfRefraction::new(1.5).unwrap();
    assert!(snells_law(n1, n2, a).is_ok());

    // Lens
    assert!(lens_maker(n2, 1.0, -1.0).is_ok());

    // Jones
    let j = JonesVector::new(CausalTensor::new(vec![Complex::new(1.0, 0.0); 2], vec![2]).unwrap());
    assert!(stokes_from_jones(&j).is_ok());

    // Beam
    let q = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap();
    let w = Wavelength::new(1e-6).unwrap();
    assert!(beam_spot_size(q, w).is_ok());

    // Diffraction
    let l = Length::new(1.0).unwrap();
    assert!(single_slit_irradiance(1.0, l, a, w).is_ok());
}
