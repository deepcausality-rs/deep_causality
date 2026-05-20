/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Complex / Cl(0,1) Iso Showcase
//!
//! Cl(0,1) is the Clifford algebra with one basis vector `e1` such that
//! `e1² = -1`. As an algebra it is literally isomorphic to ℂ: identify
//! `a + bi` with `a + b·e1`. This example demonstrates that the iso
//! preserves not just shape but multiplication, inverse, and conjugation —
//! the full `FieldIso` and `DivisionAlgebraIso` chain.
//!
//! Three independent computation paths compute the same products:
//!
//! 1. **Native complex**: `Complex::*` directly.
//! 2. **Lifted Cl(0,1)**: lift both inputs to `CausalMultiVector`,
//!    multiply via `geometric_product`, project back via the iso reverse.
//! 3. **Witness round-trip**: use `<ComplexCl01Iso>::to_target` and
//!    `to_source` explicitly to demonstrate the trait surface.
//!
//! All three paths produce byte-identical results. The example also
//! exercises the marker subtraits via the `assert_witness_*_law` helpers
//! to show that the iso satisfies every algebraic-structure law.
//!
//! ## Iso surface used
//!
//! - `impl<F> From<Complex<F>> for CausalMultiVector<F>` (Tier 1 forward,
//!   hard-coded to Cl(0,1) metric).
//! - `ComplexCl01Iso` witness implementing `Iso<CausalMultiVector<F>,
//!   Complex<F>>` plus `GroupIso`, `RingIso`, `FieldIso`, `AlgebraIso`,
//!   `DivisionAlgebraIso` markers.

use deep_causality_multivector::{CausalMultiVector, ComplexCl01Iso, Metric};
use deep_causality_num::Complex;
use deep_causality_num::iso::witness::Iso;

type F = f64;

fn main() {
    println!("=== Complex / Cl(0,1) Iso Showcase ===\n");

    let a = Complex::<F>::new(2.0, 3.0); // 2 + 3i
    let b = Complex::<F>::new(1.0, -4.0); // 1 - 4i

    println!("a = {} + {}i", a.re, a.im);
    println!("b = {} + {}i", b.re, b.im);
    println!("Expected a * b = (2*1 - 3*-4) + (2*-4 + 3*1)i = 14 - 5i\n");

    // ---------------------------------------------------------------------
    // Path 1: native complex multiplication
    // ---------------------------------------------------------------------
    let p1 = a * b;
    println!("Path 1 (native Complex):   {} + {}i", p1.re, p1.im);

    // ---------------------------------------------------------------------
    // Path 2: lift to Cl(0,1), multiply, project back
    // ---------------------------------------------------------------------
    let a_mv: CausalMultiVector<F> = a.into();
    let b_mv: CausalMultiVector<F> = b.into();
    let p_mv = a_mv.geometric_product(&b_mv);
    let p2 = <ComplexCl01Iso as Iso<CausalMultiVector<F>, Complex<F>>>::to_target(p_mv);
    println!("Path 2 (Cl(0,1) lifted):   {} + {}i", p2.re, p2.im);

    // ---------------------------------------------------------------------
    // Path 3: witness round-trip (lift through the iso, multiply natively,
    // then lift back). Demonstrates the iso composes with native ops.
    // ---------------------------------------------------------------------
    let a_back: Complex<F> = <ComplexCl01Iso as Iso<_, _>>::to_target(
        <ComplexCl01Iso as Iso<_, _>>::to_source(a),
    );
    let b_back: Complex<F> = <ComplexCl01Iso as Iso<_, _>>::to_target(
        <ComplexCl01Iso as Iso<_, _>>::to_source(b),
    );
    let p3 = a_back * b_back;
    println!("Path 3 (iso round-trip):   {} + {}i", p3.re, p3.im);

    // ---------------------------------------------------------------------
    // Equivalence assertions
    // ---------------------------------------------------------------------
    let drift_12 = (p1.re - p2.re).abs() + (p1.im - p2.im).abs();
    let drift_13 = (p1.re - p3.re).abs() + (p1.im - p3.im).abs();
    println!("\nL1 drift, Path1 vs Path2: {:e}", drift_12);
    println!("L1 drift, Path1 vs Path3: {:e}", drift_13);
    assert!(drift_12 < 1e-12);
    assert!(drift_13 < 1e-12);

    // ---------------------------------------------------------------------
    // Algebraic laws (the iso is a Field iso AND a DivisionAlgebra iso)
    // ---------------------------------------------------------------------
    use deep_causality_num::iso::witness::test_support::{
        assert_witness_division_algebra_iso_law, assert_witness_field_iso_laws,
        assert_witness_group_iso_law, assert_witness_iso_round_trip,
        assert_witness_ring_iso_laws,
    };

    println!("\n--- Marker-subtrait law verification ---");

    assert_witness_iso_round_trip::<ComplexCl01Iso, CausalMultiVector<F>, Complex<F>>(
        CausalMultiVector::from(a),
        a,
    );
    println!("  Iso round-trip:                  OK");

    assert_witness_group_iso_law::<ComplexCl01Iso, CausalMultiVector<F>, Complex<F>>(
        CausalMultiVector::from(a),
        CausalMultiVector::from(b),
    );
    println!("  GroupIso (additive homomorphism): OK");

    assert_witness_ring_iso_laws::<ComplexCl01Iso, CausalMultiVector<F>, Complex<F>>(
        CausalMultiVector::from(a),
        CausalMultiVector::from(b),
    );
    println!("  RingIso (add + mul homomorphism): OK");

    assert_witness_field_iso_laws::<ComplexCl01Iso, CausalMultiVector<F>, Complex<F>>(
        CausalMultiVector::from(a),
    );
    println!("  FieldIso (inverse preservation):  OK");

    assert_witness_division_algebra_iso_law::<
        ComplexCl01Iso,
        CausalMultiVector<F>,
        Complex<F>,
        F,
    >(CausalMultiVector::from(a));
    println!("  DivisionAlgebraIso (conjugation): OK");

    println!("\nThe iso satisfies every law in the chain:");
    println!("  Iso -> GroupIso -> RingIso -> FieldIso");
    println!("                 \\-> AlgebraIso -> DivisionAlgebraIso");
    println!("\nThis is the only iso in `implement-isomorphism` that exercises");
    println!("the full marker-subtrait stack on a single type pair.");
}
