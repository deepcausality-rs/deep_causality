/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_topology::GaugeGroup;
use deep_causality_topology::LatticeComplex;
use deep_causality_topology::LatticeGaugeField;
use deep_causality_topology::LinkVariable;
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
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
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
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let err = field.try_wilson_loop(&site, 0, 1, 0, 1);
    assert!(err.is_err());
}

#[test]
fn test_wilson_loop_identity() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let w = field.try_wilson_loop(&site, 0, 1, 2, 2).unwrap();
    // Identity loop = 1.0 (after normalization 1/N)
    assert!((w - 1.0).abs() < 1e-10);
}

#[test]
fn test_polyakov_loop_invalid_dir() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let err = field.try_polyakov_loop(&site, 99);
    assert!(err.is_err());
}

#[test]
fn test_polyakov_loop_identity() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let p = field.try_polyakov_loop(&site, 0).unwrap();
    assert!((p - 1.0).abs() < 1e-10);
}

#[test]
fn test_average_polyakov_loop_identity_is_one() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let avg = field.try_average_polyakov_loop(0).unwrap();
    assert!((avg - 1.0).abs() < 1e-10);
}

#[test]
fn test_average_polyakov_loop_empty_lattice_returns_zero() {
    // Empty-lattice path through `if count == 0 { return Ok(R::zero()) }`.
    use std::collections::HashMap;
    let lattice = Arc::new(LatticeComplex::<2, f64>::new([0, 0], [false, false]));
    let links: HashMap<_, LinkVariable<U1, Complex<f64>, f64>> = HashMap::new();
    let field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
        LatticeGaugeField::from_links_unchecked(lattice, links, 1.0, ());
    let avg = field.try_average_polyakov_loop(0).unwrap();
    assert_eq!(avg, 0.0);
}

#[test]
fn test_plaquette_action_identity_is_zero() {
    // Drives try_plaquette_action through its full happy path.
    let shape = [2, 2];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let s = field.try_plaquette_action(&[0, 0], 0, 1).unwrap();
    assert!(s.abs() < 1e-10, "identity plaquette action = 0, got {s}");
}
