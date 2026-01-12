/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_topology::GaugeGroup;
use deep_causality_topology::Lattice;
use deep_causality_topology::LatticeGaugeField;
use std::sync::Arc;

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
fn test_wilson_loop_invalid_dirs() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];

    // r_dir == t_dir
    let err = field.try_wilson_loop(&site, 0, 0, 1, 1);
    assert!(err.is_err());

    // dir out of bounds
    let err = field.try_wilson_loop(&site, 0, 99, 1, 1);
    assert!(err.is_err());
}

#[test]
fn test_wilson_loop_zero_size() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let err = field.try_wilson_loop(&site, 0, 1, 0, 1);
    assert!(err.is_err());
}

#[test]
fn test_wilson_loop_identity() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let w = field.try_wilson_loop(&site, 0, 1, 2, 2).unwrap();
    // Identity loop = 1.0 (after normalization 1/N)
    assert!((w - 1.0).abs() < 1e-10);
}

#[test]
fn test_polyakov_loop_invalid_dir() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let err = field.try_polyakov_loop(&site, 99);
    assert!(err.is_err());
}

#[test]
fn test_polyakov_loop_identity() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let p = field.try_polyakov_loop(&site, 0).unwrap();
    assert!((p - 1.0).abs() < 1e-10);
}
