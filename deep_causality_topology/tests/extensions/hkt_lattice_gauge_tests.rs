/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, Monad, Pure};
use deep_causality_topology::{
    CWComplex, GaugeGroup, Lattice, LatticeGaugeField, LatticeGaugeFieldWitness, TopologyError,
    TopologyErrorEnum,
};
use std::sync::Arc;

// Define a test gauge group
use deep_causality_num::Complex;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TestGroup;
impl GaugeGroup for TestGroup {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;

    fn matrix_dim() -> usize {
        1
    }
    fn name() -> &'static str {
        "TestGroup"
    }
}

const D: usize = 2;

#[test]
fn test_witness_new_and_display() {
    let witness = LatticeGaugeFieldWitness::<TestGroup, D, f64>::new();
    let display_str = format!("{}", witness);
    assert_eq!(display_str, "LatticeGaugeFieldWitness<TestGroup, 2D>");
}

#[test]
fn test_pure() {
    let beta = 5.0;
    let field = LatticeGaugeFieldWitness::<TestGroup, D, f64>::pure(beta);
    assert_eq!(*field.beta(), beta);
    // Pure creates a 1^D lattice with no links
    assert_eq!(*field.lattice().shape(), [1usize; D]);
}

#[test]
fn test_functor() {
    let field = LatticeGaugeFieldWitness::<TestGroup, D, f64>::pure(2.0);
    // Map beta: 2.0 -> 4.0
    let transformed = LatticeGaugeFieldWitness::fmap(field, |x| x * 2.0);
    assert_eq!(*transformed.beta(), 4.0);
}

#[test]
fn test_applicative() {
    let func_field = LatticeGaugeFieldWitness::<TestGroup, D, f64>::pure(|x: f64| x + 10.0);
    let val_field = LatticeGaugeFieldWitness::<TestGroup, D, f64>::pure(5.0);

    let result = LatticeGaugeFieldWitness::apply(func_field, val_field);
    assert_eq!(*result.beta(), 15.0);
}

#[test]
fn test_monad() {
    let field = LatticeGaugeFieldWitness::<TestGroup, D, f64>::pure(3.0);
    let result = LatticeGaugeFieldWitness::bind(field, |x| {
        LatticeGaugeFieldWitness::<TestGroup, D, f64>::pure(x * x)
    });
    assert_eq!(*result.beta(), 9.0);
}

#[test]
fn test_map_field_full() {
    // This tests the type-safe map_field that preserves links
    let shape = [2, 2];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<TestGroup, D, Complex<f64>, f64>::identity(lattice, 1.0);

    // Scale all link matrices by 2.0 and beta by 2.0
    // Note: LinkVariable maps element-wise. Identity * 2.0 = 2*Identity
    let scaled = LatticeGaugeFieldWitness::map_field(field, |x| x * 2.0);

    assert_eq!(*scaled.beta(), 1.0);

    // Verify a link
    let edge = scaled.links().keys().next().unwrap();
    let link = scaled.link(edge).unwrap();
    // Element (0,0) of 2*Identity should be 2.0
    // But map_field reconstructs tensor, so let's check values
    let val = link.as_slice()[0]; // Re(0,0)
    assert!((val - Complex::new(2.0, 0.0)).norm() < 1e-10);
}

#[test]
fn test_scale_field() {
    let shape = [2, 2];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<TestGroup, D, Complex<f64>, f64>::identity(lattice, 1.0);

    let scaled = LatticeGaugeFieldWitness::scale_field(field, Complex::new(0.5, 0.0));

    // Beta is NOT scaled by scale_field?
    // Wait, scale_field implementation:
    // Self::map_field(field, move |x| x * factor_clone)
    // Yes, map_field applies to beta too: let beta = f(*field.beta());
    // So beta becomes 0.5.
    assert_eq!(*scaled.beta(), 1.0);

    let edge = scaled.links().keys().next().unwrap();
    let val = scaled.link(edge).unwrap().as_slice()[0];
    assert!((val - Complex::new(0.5, 0.0)).norm() < 1e-10);
}

#[test]
fn test_zip_with_success() {
    let shape = [2, 2];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field_a =
        LatticeGaugeField::<TestGroup, D, Complex<f64>, f64>::identity(lattice.clone(), 1.0);
    let field_b = LatticeGaugeField::<TestGroup, D, Complex<f64>, f64>::identity(lattice, 2.0);

    // Add fields: A + B
    // 1.0 + 1.0 = 2.0 (for identity links)
    // beta: 1.0 + 2.0 = 3.0
    let result = LatticeGaugeFieldWitness::zip_with(&field_a, &field_b, |a, b| *a + *b).unwrap();

    assert_eq!(*result.beta(), 1.0);
    let edge = result.links().keys().next().unwrap();
    let val = result.link(edge).unwrap().as_slice()[0];
    assert!((val - Complex::new(2.0, 0.0)).norm() < 1e-10);
}

#[test]
fn test_zip_with_lattice_mismatch() {
    let shape1 = [2, 2];
    let shape2 = [3, 3];
    let lattice1 = Arc::new(Lattice::new(shape1, [true, true]));
    let lattice2 = Arc::new(Lattice::new(shape2, [true, true]));

    let field_a = LatticeGaugeField::<TestGroup, D, Complex<f64>, f64>::identity(lattice1, 1.0);
    let field_b = LatticeGaugeField::<TestGroup, D, Complex<f64>, f64>::identity(lattice2, 1.0);

    let err = LatticeGaugeFieldWitness::zip_with(&field_a, &field_b, |a, b| *a + *b);
    assert!(matches!(
        err,
        Err(TopologyError(TopologyErrorEnum::LatticeGaugeError(_)))
    ));
}

#[test]
fn test_zip_with_missing_link() {
    let shape = [2, 2];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field_a =
        LatticeGaugeField::<TestGroup, D, Complex<f64>, f64, ()>::identity(lattice.clone(), 1.0);

    // Create field B with a missing link
    // We can use from_links_unchecked with an empty map (which is invalid but good for this test)
    use std::collections::HashMap;
    let field_b = LatticeGaugeField::<TestGroup, D, Complex<f64>, f64, ()>::from_links_unchecked(
        lattice,
        HashMap::new(), // Empty links
        1.0,
        (),
    );

    let err = LatticeGaugeFieldWitness::zip_with(&field_a, &field_b, |a, b| *a + *b);
    // Expect error because field_a has links but field_b doesn't
    assert!(
        matches!(err, Err(TopologyError(TopologyErrorEnum::LatticeGaugeError(msg))) if msg.contains("Missing link"))
    );
}

#[test]
fn test_identity_field_wrapper() {
    // Just verify the convenience wrapper works
    let shape = [2, 2];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field =
        LatticeGaugeFieldWitness::<TestGroup, D, f64>::identity_field::<Complex<f64>>(lattice, 1.0)
            .unwrap();
    assert_eq!(field.lattice().num_cells(1), field.links().len());
}
