/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CellComplexWitness` and `LatticeComplexWitness`.
//!
//! Both witnesses used to implement `HKT` and stop, and the types their `Type<T>` named were not
//! exported, so a caller could name `<CellComplexWitness<C> as HKT>::Type<T>` and receive a type it
//! could not construct or use. They now carry `Functor` and `Foldable`, and both field types are
//! public. These tests cover the operations and the laws they have to satisfy.
//!
//! The absent instances are covered too, in the sense that matters: `fmap` is asserted to preserve
//! the complex, which is the property that a fabricating `Pure` would destroy.

use deep_causality_haft::{Foldable, Functor};
use deep_causality_topology::utils_tests::LawRng;
use deep_causality_topology::{
    CellComplex, CellComplexWitness, CellField, LatticeComplex, LatticeComplexWitness,
    LatticeField, Simplex,
};
use std::sync::Arc;

type CW = CellComplexWitness<Simplex>;
type LW = LatticeComplexWitness<2, f64>;

/// A triangle with its edges and vertices, closed under boundary.
fn triangle_complex() -> Arc<CellComplex<Simplex>> {
    let cells = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
        Simplex::new(vec![0, 1]),
        Simplex::new(vec![0, 2]),
        Simplex::new(vec![1, 2]),
        Simplex::new(vec![0, 1, 2]),
    ];
    Arc::new(CellComplex::from_cells(cells))
}

fn lattice() -> Arc<LatticeComplex<2, f64>> {
    Arc::new(LatticeComplex::new([3, 3], [false, false]))
}

// ----------------------------------------------------------------------------
// CellComplexWitness
// ----------------------------------------------------------------------------

#[test]
fn cell_field_constructor_and_accessors() {
    let field = CellField::new(triangle_complex(), vec![1.0f64, 2.0, 3.0]);
    assert_eq!(field.len(), 3);
    assert!(!field.is_empty());
    assert_eq!(field.values(), &[1.0, 2.0, 3.0]);
    assert!(Arc::ptr_eq(field.complex(), field.complex()));

    let empty: CellField<Simplex, f64> = CellField::new(triangle_complex(), vec![]);
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
}

#[test]
fn cell_field_fmap_maps_values() {
    let field = CellField::new(triangle_complex(), vec![1.0f64, 2.0, 3.0]);
    let doubled = <CW as Functor<CW>>::fmap(field, |x| x * 2.0);
    assert_eq!(doubled.values(), &[2.0, 4.0, 6.0]);
}

#[test]
fn cell_field_fmap_changes_the_value_type() {
    let field = CellField::new(triangle_complex(), vec![1.9f64, 2.9, 3.1]);
    let labels = <CW as Functor<CW>>::fmap(field, |x| format!("{:.1}", x));
    assert_eq!(
        labels.values(),
        &["1.9".to_string(), "2.9".to_string(), "3.1".to_string()]
    );
    // A field of strings is legitimate, which the witness allows by placing no bound on the element.
}

#[test]
fn cell_field_fmap_preserves_the_complex() {
    // The property a fabricating `Pure` would break, and the reason there is none.
    let complex = triangle_complex();
    let field = CellField::new(Arc::clone(&complex), vec![1.0f64, 2.0, 3.0]);
    let mapped = <CW as Functor<CW>>::fmap(field, |x| x + 1.0);
    assert!(
        Arc::ptr_eq(mapped.complex(), &complex),
        "fmap must carry the complex across, not rebuild it"
    );
}

#[test]
fn cell_field_functor_identity() {
    let mut rng = LawRng::new(0xCE11);
    for n in [0usize, 1, 3, 8] {
        let values = rng.scalars(n, 5.0);
        let field = CellField::new(triangle_complex(), values);
        let mapped = <CW as Functor<CW>>::fmap(field.clone(), |x| x);
        assert_eq!(mapped, field, "fmap(id, c) != c at n={n}");
    }
}

#[test]
fn cell_field_functor_composition() {
    let mut rng = LawRng::new(0xCE11 ^ 1);
    for _ in 0..16 {
        let values = rng.well_scaled_vec(4, 5.0);
        let (p, q) = (rng.well_scaled(3.0), rng.well_scaled(3.0));
        let f = move |x: f64| x * p;
        let g = move |x: f64| x + q;

        let field = CellField::new(triangle_complex(), values.clone());
        let lhs = <CW as Functor<CW>>::fmap(<CW as Functor<CW>>::fmap(field, f), g);

        let field2 = CellField::new(triangle_complex(), values);
        let rhs = <CW as Functor<CW>>::fmap(field2, move |x| g(f(x)));

        for (a, b) in lhs.values().iter().zip(rhs.values()) {
            assert!((a - b).abs() < 1e-9, "fmap(g) . fmap(f) != fmap(g . f)");
        }
    }
}

#[test]
fn cell_field_fold_accumulates_every_value() {
    let field = CellField::new(triangle_complex(), vec![1.0f64, 2.0, 3.0, 4.0]);
    let sum = <CW as Foldable<CW>>::fold(field, 0.0, |acc, x| acc + x);
    assert_eq!(sum, 10.0);

    let empty: CellField<Simplex, f64> = CellField::new(triangle_complex(), vec![]);
    let zero = <CW as Foldable<CW>>::fold(empty, 0.0, |acc, x| acc + x);
    assert_eq!(zero, 0.0, "folding an empty field yields the initial value");
}

#[test]
fn cell_field_fold_can_change_the_accumulator_type() {
    let field = CellField::new(triangle_complex(), vec![1u8, 2, 3]);
    let joined = <CW as Foldable<CW>>::fold(field, String::new(), |mut acc, x| {
        acc.push_str(&x.to_string());
        acc
    });
    assert_eq!(joined, "123");
}

// ----------------------------------------------------------------------------
// LatticeComplexWitness
// ----------------------------------------------------------------------------

#[test]
fn lattice_field_constructor_and_accessors() {
    let field = LatticeField::new(lattice(), vec![1.0f64, 2.0]);
    assert_eq!(field.len(), 2);
    assert!(!field.is_empty());
    assert_eq!(field.values(), &[1.0, 2.0]);
    assert_eq!(field.lattice().shape(), &[3, 3]);
}

#[test]
fn lattice_field_fmap_maps_values_and_keeps_the_lattice() {
    let lat = lattice();
    let field = LatticeField::new(Arc::clone(&lat), vec![1.0f64, 2.0, 3.0]);
    let scaled = <LW as Functor<LW>>::fmap(field, |x| x * 10.0);
    assert_eq!(scaled.values(), &[10.0, 20.0, 30.0]);
    assert!(
        Arc::ptr_eq(scaled.lattice(), &lat),
        "fmap must carry the lattice across, not rebuild it"
    );
}

#[test]
fn lattice_field_functor_identity() {
    let mut rng = LawRng::new(0x1A77);
    for n in [0usize, 1, 4, 9] {
        let field = LatticeField::new(lattice(), rng.scalars(n, 5.0));
        let mapped = <LW as Functor<LW>>::fmap(field.clone(), |x| x);
        assert_eq!(mapped, field, "fmap(id, f) != f at n={n}");
    }
}

#[test]
fn lattice_field_functor_composition() {
    let mut rng = LawRng::new(0x1A77 ^ 1);
    for _ in 0..16 {
        let values = rng.well_scaled_vec(5, 4.0);
        let (p, q) = (rng.well_scaled(2.0), rng.well_scaled(2.0));
        let f = move |x: f64| x * p;
        let g = move |x: f64| x - q;

        let lhs = <LW as Functor<LW>>::fmap(
            <LW as Functor<LW>>::fmap(LatticeField::new(lattice(), values.clone()), f),
            g,
        );
        let rhs = <LW as Functor<LW>>::fmap(LatticeField::new(lattice(), values), move |x| g(f(x)));

        for (a, b) in lhs.values().iter().zip(rhs.values()) {
            assert!((a - b).abs() < 1e-9, "fmap(g) . fmap(f) != fmap(g . f)");
        }
    }
}

#[test]
fn lattice_field_fold_accumulates_every_value() {
    let field = LatticeField::new(lattice(), vec![2.0f64, 4.0, 6.0]);
    let sum = <LW as Foldable<LW>>::fold(field, 0.0, |acc, x| acc + x);
    assert_eq!(sum, 12.0);
}

#[test]
fn lattice_field_fmap_changes_the_value_type() {
    let field = LatticeField::new(lattice(), vec![1.0f64, 2.0]);
    let flags = <LW as Functor<LW>>::fmap(field, |x| x > 1.5);
    assert_eq!(flags.values(), &[false, true]);
}
