/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Dual/Dual.lean`.
//!
//! Lean proves these laws for all inputs; each test pins the crate's real `Dual` type to the
//! same statement at representative inputs. The `THEOREM_MAP` ids match `lean/THEOREM_MAP.md`.

use deep_causality_num_dual::Dual;

/// THEOREM_MAP: dual.comm_ring.mul_comm
#[test]
fn test_mul_comm() {
    // Multiplication commutes: a·b = b·a.
    let a = Dual::new(2.0_f64, 3.0);
    let b = Dual::new(5.0, 7.0);
    assert_eq!(a * b, b * a);
}

/// THEOREM_MAP: dual.eps_sq_zero
#[test]
fn test_eps_sq_zero() {
    // ε = 0 + 1·ε is the nilpotent unit: ε·ε = 0 in both components.
    let eps = Dual::new(0.0_f64, 1.0);
    let sq = eps * eps;
    assert_eq!(sq.value(), 0.0);
    assert_eq!(sq.derivative(), 0.0);
}

/// THEOREM_MAP: dual.real_projection.add
#[test]
fn test_real_projection_add() {
    // The real projection is additive: value(a+b) = value(a)+value(b).
    let a = Dual::new(2.0_f64, 3.0);
    let b = Dual::new(5.0, 7.0);
    assert_eq!((a + b).value(), a.value() + b.value());
}

/// THEOREM_MAP: dual.real_projection.mul
#[test]
fn test_real_projection_mul() {
    // The real projection is multiplicative: value(a·b) = value(a)·value(b).
    let a = Dual::new(2.0_f64, 3.0);
    let b = Dual::new(5.0, 7.0);
    assert_eq!((a * b).value(), a.value() * b.value());
}

/// THEOREM_MAP: dual.leibniz.product_rule
#[test]
fn test_leibniz_product_rule() {
    // Forward-mode AD Leibniz rule:
    // derivative(a·b) = value(a)·derivative(b) + derivative(a)·value(b).
    let a = Dual::new(2.0_f64, 3.0);
    let b = Dual::new(5.0, 7.0);
    assert_eq!(
        (a * b).derivative(),
        a.value() * b.derivative() + a.derivative() * b.value()
    );
}

/// THEOREM_MAP: dual.not_field.zero_divisor
#[test]
fn test_not_field_zero_divisor() {
    // ε is a nonzero zero divisor, so `Dual` is a commutative ring but not a field.
    let eps = Dual::new(0.0_f64, 1.0);
    let zero = Dual::new(0.0_f64, 0.0);

    // ε ≠ 0: its derivative part is 1.
    assert_ne!(eps, zero);
    assert_eq!(eps.derivative(), 1.0);

    // Yet ε·ε = 0, a nonzero element whose square vanishes.
    assert_eq!(eps * eps, zero);
}
