/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, Monad, Pure};
use deep_causality_topology::{Lattice, LatticeGaugeField, LatticeGaugeFieldWitness, SU2};
use std::sync::Arc;

fn create_test_lattice() -> Arc<Lattice<4>> {
    Arc::new(Lattice::new([2, 2, 2, 2], [true; 4]))
}

#[test]
fn test_hkt_pure_and_functor() {
    // Pure: Lift value into field
    let beta = 2.5f64;
    let field = LatticeGaugeFieldWitness::<SU2, 4>::pure(beta);

    assert_eq!(*field.beta(), 2.5);
    assert_eq!(field.num_links(), 0); // Minimal context has no links in the map

    // Functor: Map over beta
    let mapped = LatticeGaugeFieldWitness::<SU2, 4>::fmap(field, |b| b * 2.0);
    assert_eq!(*mapped.beta(), 5.0);
}

#[test]
fn test_hkt_monad() {
    let beta = 3.0f64;
    let field = LatticeGaugeFieldWitness::<SU2, 4>::pure(beta);

    // Monad: Chain transformation
    let chained = LatticeGaugeFieldWitness::<SU2, 4>::bind(field, |b| {
        LatticeGaugeFieldWitness::<SU2, 4>::pure(b + 1.0)
    });

    assert_eq!(*chained.beta(), 4.0);
}

#[test]
fn test_hkt_applicative() {
    // Applicative: Apply wrapped function to wrapped value
    let val_field = LatticeGaugeFieldWitness::<SU2, 4>::pure(2.0f64);
    let func_field = LatticeGaugeFieldWitness::<SU2, 4>::pure(|x: f64| x * 3.0);

    let result = LatticeGaugeFieldWitness::<SU2, 4>::apply(func_field, val_field);

    assert_eq!(*result.beta(), 6.0);
}

#[test]
fn test_map_field_full() {
    let lattice = create_test_lattice();
    let beta = 1.0f64;

    // Create random field
    let mut rng = deep_causality_rand::rng();
    let field = LatticeGaugeField::<SU2, 4, f64>::random(lattice, beta, &mut rng);

    // Map over all scalars: x -> x * 2.0
    let scaled = LatticeGaugeFieldWitness::<SU2, 4>::map_field(field.clone(), |x| x * 2.0);

    assert_eq!(*scaled.beta(), 2.0);

    // Check first link trace scaled
    let binding = field.links();
    let (cell, link) = binding.iter().next().unwrap();
    let scaled_link = scaled.link(cell).unwrap();

    // Trace should be scaled by 2.0
    let tr_orig = link.trace();
    let tr_scaled = scaled_link.trace();
    assert!((tr_scaled - tr_orig * 2.0).abs() < 1e-10);
}

#[test]
fn test_zip_with() {
    let lattice = create_test_lattice();
    let beta = 1.0f64;

    let mut rng = deep_causality_rand::rng();
    let field_a = LatticeGaugeField::<SU2, 4, f64>::random(lattice.clone(), beta, &mut rng);
    let field_b = LatticeGaugeField::<SU2, 4, f64>::random(lattice, beta, &mut rng);

    // Combine: a + b
    let sum = LatticeGaugeFieldWitness::<SU2, 4>::zip_with(&field_a, &field_b, |a, b| a + b)
        .expect("Zip failed");

    assert_eq!(*sum.beta(), 2.0);

    // Check first link
    let binding = field_a.links();
    let (cell, link_a) = binding.iter().next().unwrap();
    let link_b = field_b.link(cell).unwrap();
    let link_sum = sum.link(cell).unwrap();

    // Element-wise sum check
    let val_a = link_a.matrix().as_slice()[0];
    let val_b = link_b.matrix().as_slice()[0];
    let val_sum = link_sum.matrix().as_slice()[0];

    assert!((val_sum - (val_a + val_b)).abs() < 1e-10);
}
