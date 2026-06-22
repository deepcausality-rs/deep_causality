/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{FourMomentum, Hadron};

#[test]
fn test_hadron_properties() {
    let p = FourMomentum::<f64>::new(5.0, 3.0, 4.0, 0.0); // E=5, px=3, py=4, pz=0
    // pt = sqrt(3^2+4^2) = 5. Mass should be 0 (lightlike) ideally, but here E^2 - p^2 = 25 - 25 = 0.
    let h = Hadron::<f64>::new(211, p); // 211 = pi+

    assert_eq!(h.pdg_id(), 211);
    assert_eq!(h.energy(), 5.0);
    assert_eq!(h.pt(), 5.0);
    assert_eq!(h.mass(), 0.0); // Massless in this example
    assert_eq!(h.momentum(), p);
}

#[test]
fn test_hadron_rapidity() {
    // E=10, pz=8 => rapidity = 0.5*ln(18/2) > 0.
    let p = FourMomentum::<f64>::new(10.0, 0.0, 0.0, 8.0);
    let h = Hadron::<f64>::new(211, p);
    let expected = 0.5_f64 * (18.0_f64 / 2.0_f64).ln();
    assert!((h.rapidity() - expected).abs() < 1e-10);
}
