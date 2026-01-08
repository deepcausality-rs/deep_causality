/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_physics::{AVOGADRO_CONSTANT, BOLTZMANN_CONSTANT, STEFAN_BOLTZMANN_CONSTANT};

#[test]
fn test_thermodynamics_constants_sanity() {
    // Boltzmann Constant ~ 1.380e-23 J/K
    assert!((BOLTZMANN_CONSTANT - 1.380_649e-23).abs() < 1e-29); // Exact in SI
    // Stefan-Boltzmann ~ 5.670e-8
    assert!((STEFAN_BOLTZMANN_CONSTANT - 5.670_374_419e-8).abs() < 1e-16);
    // Avogadro ~ 6.022e23
    assert!((AVOGADRO_CONSTANT - 6.022_140_76e23).abs() < 1e15); // Exact
}
