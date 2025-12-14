/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Density, PhysicalField, alfven_speed, magnetic_pressure,
    magnetic_reconnection_rate,
};

#[test]
fn test_wrappers_mhd() {
    // Alfven
    let b = PhysicalField::new(
        CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap(),
    );
    let rho = Density::new(1.0).unwrap();
    assert!(alfven_speed(&b, &rho, 1.0).is_ok());

    // Magnetic Pressure
    assert!(magnetic_pressure(&b, 1.0).is_ok());

    // Reconnection
    let va = alfven_speed(&b, &rho, 1.0)
        .value()
        .clone()
        .into_value()
        .unwrap();
    assert!(magnetic_reconnection_rate(va, 100.0).is_ok());
}
