/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Activity, AmountOfSubstance, EnergyDensity, HalfLife};

#[test]
fn test_nuclear_scalars_traits() {
    let a = AmountOfSubstance::<f64>::new(1.0).unwrap();
    assert_eq!(a, a.clone());
    let _ = format!("{:?}", a);

    let h = HalfLife::<f64>::new(100.0).unwrap();
    assert_eq!(h, h.clone());
    let _ = format!("{:?}", h);

    let act = Activity::<f64>::new(1.0).unwrap();
    assert_eq!(act, act.clone());
    let _ = format!("{:?}", act);

    let ed = EnergyDensity::<f64>::new(1.0).unwrap();
    assert_eq!(ed, ed.clone());
    let _ = format!("{:?}", ed);
}
