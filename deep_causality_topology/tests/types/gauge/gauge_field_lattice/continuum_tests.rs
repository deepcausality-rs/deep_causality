/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_topology::GaugeGroup;
use deep_causality_topology::Lattice;
use deep_causality_topology::LatticeGaugeField;
use std::sync::Arc;

// Define a test gauge group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct U1;

impl GaugeGroup for U1 {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;

    fn matrix_dim() -> usize {
        1
    }
    fn name() -> &'static str {
        "U1"
    }
}

#[test]
fn test_field_strength_diagonal() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    // F_mu_mu should be zero
    let f_00 = field.try_field_strength(&site, 0, 0).unwrap();

    // Check if it's zero
    let data = f_00.as_slice();
    assert_eq!(data[0], Complex::new(0.0, 0.0));
}

#[test]
fn test_field_strength_calculation() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let f_01 = field.try_field_strength(&site, 0, 1).unwrap();

    // For identity field, plaquette is identity.
    // F_01 ~ (U - U_dag)/2 = (1 - 1)/2 = 0
    let data = f_01.as_slice();
    assert!((data[0]).norm() < 1e-10);
}

#[test]
fn test_topological_charge_density_low_dim() {
    let shape = [4, 4]; // 2D
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let q = field.try_topological_charge_density(&site).unwrap();

    // Should be exactly 0.0 for D < 4
    assert_eq!(q, 0.0);
}

#[test]
fn test_topological_charge_density_4d() {
    let shape = [4, 4, 4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true; 4]));
    let field = LatticeGaugeField::<U1, 4, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0, 0, 0];
    let q = field.try_topological_charge_density(&site).unwrap();

    // For identity field, F=0, so q=0
    assert!((q).abs() < 1e-10);

    let total_q = field.try_topological_charge().unwrap();
    assert!((total_q).abs() < 1e-10);
}
