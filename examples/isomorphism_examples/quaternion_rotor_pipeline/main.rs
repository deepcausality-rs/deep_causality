/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Quaternion / Cl(3,0)-Rotor Iso Showcase
//!
//! Rotate a 3D vector through three successive Clifford rotors. The example
//! contrasts two ways to construct each rotor:
//!
//! - **BEFORE**: pack scalar and bivector coefficients into an 8-element
//!   `Vec<f64>` by hand, remembering which slot corresponds to which basis
//!   blade. Cl(3,0) basis order: `[1, e1, e2, e3, e12, e13, e23, e123]`.
//! - **AFTER**: build a `Quaternion`, then lift to `CausalMultiVector` via
//!   `.into()`. East-coast convention: `i = e2e3`, `j = e3e1`, `k = e1e2`.
//!
//! Both paths produce byte-identical rotated vectors. The AFTER path saves
//! ~8 LoC per rotor and removes the basis-index-bookkeeping footgun.
//!
//! ## Iso surface used
//!
//! - `impl<F> From<Quaternion<F>> for CausalMultiVector<F>` (Tier 1 forward,
//!   from the `implement-isomorphism` change).
//! - `QuaternionRotorIso` witness implementing
//!   `Iso<CausalMultiVector<F>, Quaternion<F>>` (Tier 2 reverse for the
//!   always-valid rotor path).
//!
//! ## Scenario
//!
//! Take the unit-x vector `[1, 0, 0]` and apply three rotations:
//! 1. 90° around the `e1^e2` plane (XY rotation) — rotates X into Y.
//! 2. 45° around the `e2^e3` plane (YZ rotation).
//! 3. 30° around the `e3^e1` plane (ZX rotation).
//!
//! Print the intermediate and final vectors. Verify both paths agree.

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::Quaternion;

type F = f64;

fn main() {
    println!("=== Quaternion / Cl(3,0)-Rotor Iso Showcase ===\n");

    let v0: [F; 3] = [1.0, 0.0, 0.0];
    println!("Starting vector: {:?}\n", v0);

    let metric = Metric::Euclidean(3);
    let v_mv = lift_vector(&v0, metric);

    // ---------------------------------------------------------------------
    // BEFORE: hand-packed coefficient vectors
    // ---------------------------------------------------------------------
    println!("--- BEFORE: manual coefficient packing ---");
    let (r1_before, r1_rev_before) = rotor_before_xy(std::f64::consts::FRAC_PI_2, metric);
    let (r2_before, r2_rev_before) = rotor_before_yz(std::f64::consts::FRAC_PI_4, metric);
    let (r3_before, r3_rev_before) = rotor_before_zx(std::f64::consts::FRAC_PI_6, metric);

    let v1_before = apply_rotor(&v_mv, &r1_before, &r1_rev_before);
    let v2_before = apply_rotor(&v1_before, &r2_before, &r2_rev_before);
    let v3_before = apply_rotor(&v2_before, &r3_before, &r3_rev_before);

    println!(
        "  after rotor 1 (XY 90°):  {:?}",
        extract_vector(&v1_before)
    );
    println!(
        "  after rotor 2 (YZ 45°):  {:?}",
        extract_vector(&v2_before)
    );
    println!(
        "  after rotor 3 (ZX 30°):  {:?}",
        extract_vector(&v3_before)
    );

    // ---------------------------------------------------------------------
    // AFTER: quaternion construction + iso lift
    // ---------------------------------------------------------------------
    println!("\n--- AFTER: quaternion-built rotors ---");
    let (r1_after, r1_rev_after) = rotor_after_xy(std::f64::consts::FRAC_PI_2);
    let (r2_after, r2_rev_after) = rotor_after_yz(std::f64::consts::FRAC_PI_4);
    let (r3_after, r3_rev_after) = rotor_after_zx(std::f64::consts::FRAC_PI_6);

    let v1_after = apply_rotor(&v_mv, &r1_after, &r1_rev_after);
    let v2_after = apply_rotor(&v1_after, &r2_after, &r2_rev_after);
    let v3_after = apply_rotor(&v2_after, &r3_after, &r3_rev_after);

    println!("  after rotor 1 (XY 90°):  {:?}", extract_vector(&v1_after));
    println!("  after rotor 2 (YZ 45°):  {:?}", extract_vector(&v2_after));
    println!("  after rotor 3 (ZX 30°):  {:?}", extract_vector(&v3_after));

    // ---------------------------------------------------------------------
    // Equivalence check
    // ---------------------------------------------------------------------
    let v_before = extract_vector(&v3_before);
    let v_after = extract_vector(&v3_after);
    let drift: F = v_before
        .iter()
        .zip(v_after.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();

    println!("\nL1 drift between BEFORE and AFTER paths: {:e}", drift);
    assert!(drift < 1e-12, "iso path diverged from manual path");
    println!("Both paths produce the same result.\n");

    // ---------------------------------------------------------------------
    // Line count summary
    // ---------------------------------------------------------------------
    println!("Per-rotor LoC, BEFORE:  ~9 lines (vec![] + index assignments + unwrap)");
    println!("Per-rotor LoC, AFTER:   ~2 lines (Quaternion::new + .into())");
}

// =============================================================================
// BEFORE: hand-packed rotors
// =============================================================================

/// XY rotor: `r = cos(θ/2) - sin(θ/2) e1^e2`. e1^e2 is index 3 in the Cl(3,0)
/// basis. The reverse rotor flips the bivector sign.
fn rotor_before_xy(theta: F, metric: Metric) -> (CausalMultiVector<F>, CausalMultiVector<F>) {
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    // Cl(3,0) basis: [1, e1, e2, e3, e12, e13, e23, e123]
    let mut r = vec![0.0; 8];
    r[0] = c;
    r[4] = -s; // -sin(θ/2) on e12
    let mut r_rev = vec![0.0; 8];
    r_rev[0] = c;
    r_rev[4] = s;
    (
        CausalMultiVector::new(r, metric).unwrap(),
        CausalMultiVector::new(r_rev, metric).unwrap(),
    )
}

/// YZ rotor: `r = cos(θ/2) - sin(θ/2) e2^e3`. e2^e3 is index 6.
fn rotor_before_yz(theta: F, metric: Metric) -> (CausalMultiVector<F>, CausalMultiVector<F>) {
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    let mut r = vec![0.0; 8];
    r[0] = c;
    r[6] = -s;
    let mut r_rev = vec![0.0; 8];
    r_rev[0] = c;
    r_rev[6] = s;
    (
        CausalMultiVector::new(r, metric).unwrap(),
        CausalMultiVector::new(r_rev, metric).unwrap(),
    )
}

/// ZX rotor: `r = cos(θ/2) - sin(θ/2) e3^e1`. e3^e1 is index 5 (= e1^e3 with
/// a sign — see basis-index commentary). The reader has to know the sign
/// convention.
fn rotor_before_zx(theta: F, metric: Metric) -> (CausalMultiVector<F>, CausalMultiVector<F>) {
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    let mut r = vec![0.0; 8];
    r[0] = c;
    r[5] = -s;
    let mut r_rev = vec![0.0; 8];
    r_rev[0] = c;
    r_rev[5] = s;
    (
        CausalMultiVector::new(r, metric).unwrap(),
        CausalMultiVector::new(r_rev, metric).unwrap(),
    )
}

// =============================================================================
// AFTER: quaternion-built rotors via iso
// =============================================================================

/// XY rotor: quaternion `(c, 0, 0, -s)`. The k component (e1^e2) carries the
/// rotation amount; e2^e3 (i) and e3^e1 (j) are zero.
fn rotor_after_xy(theta: F) -> (CausalMultiVector<F>, CausalMultiVector<F>) {
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    // East-coast convention: w + xi + yj + zk where i=e23, j=e31, k=e12.
    let q: CausalMultiVector<F> = Quaternion::new(c, 0.0, 0.0, -s).into();
    let q_rev: CausalMultiVector<F> = Quaternion::new(c, 0.0, 0.0, s).into();
    (q, q_rev)
}

/// YZ rotor: quaternion `(c, -s, 0, 0)`. The i component (e2^e3) carries the
/// rotation amount.
fn rotor_after_yz(theta: F) -> (CausalMultiVector<F>, CausalMultiVector<F>) {
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    let q: CausalMultiVector<F> = Quaternion::new(c, -s, 0.0, 0.0).into();
    let q_rev: CausalMultiVector<F> = Quaternion::new(c, s, 0.0, 0.0).into();
    (q, q_rev)
}

/// ZX rotor: quaternion `(c, 0, -s, 0)`. The j component (e3^e1) carries the
/// rotation amount.
fn rotor_after_zx(theta: F) -> (CausalMultiVector<F>, CausalMultiVector<F>) {
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    let q: CausalMultiVector<F> = Quaternion::new(c, 0.0, -s, 0.0).into();
    let q_rev: CausalMultiVector<F> = Quaternion::new(c, 0.0, s, 0.0).into();
    (q, q_rev)
}

// =============================================================================
// Helpers: lift / apply / extract (unchanged between BEFORE and AFTER paths)
// =============================================================================

/// Embed a 3D vector as a grade-1 multivector. Coefficients on e1, e2, e3 are
/// `v[0], v[1], v[2]`; all other blades are zero.
fn lift_vector(v: &[F; 3], metric: Metric) -> CausalMultiVector<F> {
    let coeffs = vec![0.0, v[0], v[1], v[2], 0.0, 0.0, 0.0, 0.0];
    CausalMultiVector::new(coeffs, metric).unwrap()
}

/// Apply `r v r_rev` (Clifford sandwich product).
fn apply_rotor(
    v: &CausalMultiVector<F>,
    r: &CausalMultiVector<F>,
    r_rev: &CausalMultiVector<F>,
) -> CausalMultiVector<F> {
    r.geometric_product(v).geometric_product(r_rev)
}

/// Pull the grade-1 coefficients back out as a `[F; 3]`.
fn extract_vector(mv: &CausalMultiVector<F>) -> [F; 3] {
    let d = mv.data();
    [d[1], d[2], d[3]]
}
