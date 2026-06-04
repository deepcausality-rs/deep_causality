/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{
    Associative, Commutative, CommutativeRing, Distributive, Dual, Module, One, Zero,
};

#[test]
fn test_new_and_accessors() {
    let d = Dual::new(2.0_f64, 3.0);
    assert_eq!(d.value(), 2.0);
    assert_eq!(d.derivative(), 3.0);
    assert_eq!(d.re, 2.0);
    assert_eq!(d.du, 3.0);
}

#[test]
fn test_constant_has_zero_derivative() {
    let d = Dual::constant(5.0_f64);
    assert_eq!(d.value(), 5.0);
    assert_eq!(d.derivative(), 0.0);
}

#[test]
fn test_variable_is_the_seed() {
    let d = Dual::variable(7.0_f64);
    assert_eq!(d.value(), 7.0);
    assert_eq!(d.derivative(), 1.0);
}

#[test]
fn test_copy_clone_eq() {
    let d = Dual::new(1.0_f64, 2.0);
    let e = d; // Copy
    assert_eq!(d, e);
    assert_eq!(d, d); // PartialEq reflexive
    assert_eq!(d, Dual::new(1.0, 2.0));
    assert_ne!(d, Dual::new(1.0, 3.0));
}

#[test]
fn test_zero_and_one() {
    let z: Dual<f64> = Dual::zero();
    assert!(z.is_zero());
    assert_eq!(z.value(), 0.0);
    assert_eq!(z.derivative(), 0.0);

    let o: Dual<f64> = Dual::one();
    assert!(o.is_one());
    assert_eq!(o.value(), 1.0);
    assert_eq!(o.derivative(), 0.0);
}

#[test]
fn test_ring_identities() {
    let d = Dual::new(3.0_f64, 2.0);
    assert_eq!(d + Dual::zero(), d);
    assert_eq!(d * Dual::one(), d);
    assert_eq!(d * Dual::zero(), Dual::zero());
}

#[test]
fn test_epsilon_is_nilpotent() {
    // ε = 0 + 1·ε, and ε² = 0 — the defining property of dual numbers.
    let eps = Dual::new(0.0_f64, 1.0);
    let sq = eps * eps;
    assert_eq!(sq, Dual::<f64>::zero());
    assert_eq!(sq.value(), 0.0);
    assert_eq!(sq.derivative(), 0.0);
}

#[test]
fn test_partial_ord_is_lexicographic() {
    assert!(Dual::new(1.0_f64, 0.0) < Dual::new(2.0, 0.0));
    assert!(Dual::new(1.0_f64, 1.0) > Dual::new(1.0, 0.0));
}

#[test]
fn test_algebra_typing_commutative_ring_with_markers() {
    // Dual is a commutative ring carrying all three property markers and a module over T.
    // It is deliberately NOT a Field/RealField (ε is a zero divisor; it implements neither
    // Field nor DivAssign), which holds by construction — there is no such impl to satisfy
    // a `T: Field` bound.
    fn assert_comm_ring<T: CommutativeRing>() {}
    fn assert_markers<T: Associative + Commutative + Distributive>() {}
    fn assert_module<T: Module<f64>>() {}
    assert_comm_ring::<Dual<f64>>();
    assert_markers::<Dual<f64>>();
    assert_module::<Dual<f64>>();
}
