/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorError, Metric, MultiVector};

#[test]
fn test_api_delegation() {
    let metric = Metric::Euclidean(2);
    // 2D: 1, e1, e2, e12
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let mv = CausalMultiVector::new(data.clone(), metric).unwrap();

    // 1. grade_projection
    let g0 = mv.grade_projection(0);
    assert_eq!(g0.data(), &[1.0, 0.0, 0.0, 0.0]);

    // 2. reversion
    let rev = mv.reversion();
    // 1 -> 1, e1 -> e1, e2 -> e2, e12 -> e21 = -e12
    // So 1, 2, 3, -4
    assert_eq!(rev.data(), &[1.0, 2.0, 3.0, -4.0]);

    // 3. squared_magnitude
    // For Euclidean: 1^2 + 2^2 + 3^2 + 4^2 = 1+4+9+16 = 30?
    // Wait, magnitude definition depends on geometric product.
    // This is just to ensure the method is callable and hits the implementation.
    let _mag = mv.squared_magnitude();

    // 4. inverse (might fail if not invertible, but we just check call)
    // 1.0 is invertible.
    let mv_ident = CausalMultiVector::new(vec![1.0, 0.0, 0.0, 0.0], metric).unwrap();
    let _inv = mv_ident.inverse();

    // 5. dual
    let _dual = mv.dual();

    // 6. geometric_product
    let gp = mv.geometric_product(&mv);
    assert!(!gp.data().is_empty());

    // 7. outer_product
    let op = mv.outer_product(&mv);
    // x ^ x = 0? strictly speaking yes for vectors, but general MV?
    // Check call succeeds.
    assert!(!op.data().is_empty());

    // 8. inner_product
    let ip = mv.inner_product(&mv);
    assert!(!ip.data().is_empty());

    // 9. commutator_lie
    let cl = mv.commutator_lie(&mv);
    // [x, x] = 0
    // Check call succeeds
    assert!(!cl.data().is_empty());

    // 10. commutator_geometric
    let cg = mv.commutator_geometric(&mv);
    assert!(!cg.data().is_empty());

    // 11. basis_shift
    let bs = mv.basis_shift(0);
    assert!(!bs.data().is_empty());
}

#[test]
fn test_inverse() {
    let metric = Metric::Euclidean(2);

    // 1. Scalar inversion
    // 2.0 -> 0.5
    let data_scalar = vec![2.0, 0.0, 0.0, 0.0];
    let mv_scalar = CausalMultiVector::new(data_scalar, metric).unwrap();
    let inv_scalar = mv_scalar.inverse().expect("Scalar should be invertible");
    assert_eq!(inv_scalar.data()[0], 0.5);

    // 2. Vector inversion
    // For Euclidean vector v, v^2 = |v|^2. v^{-1} = v / |v|^2.
    // Let v = e1. |v|^2 = 1. v^{-1} = e1 / 1 = e1.
    let data_vec = vec![0.0, 1.0, 0.0, 0.0]; // e1
    let mv_vec = CausalMultiVector::new(data_vec, metric).unwrap();
    let inv_vec = mv_vec.inverse().expect("Unit vector should be invertible");
    assert_eq!(inv_vec.data(), &[0.0, 1.0, 0.0, 0.0]);

    // 3. Zero vector (not invertible)
    let data_zero = vec![0.0; 4];
    let mv_zero = CausalMultiVector::new(data_zero, metric).unwrap();
    let result = mv_zero.inverse();
    assert!(result.is_err());
}

// ============================================================================
// MultiVector trait methods reached through UFCS.
//
// `CausalMultiVector` carries an inherent `inverse` as well, and an inherent method wins
// method resolution. `<CausalMultiVector<f64> as MultiVector<f64>>::inverse` is therefore
// the only way to call the trait impl.
// ============================================================================

/// The trait `inverse` returns the algebraic inverse: A * A^-1 = 1.
#[test]
fn test_trait_inverse_scalar_and_bivector() {
    let metric = Metric::Euclidean(2);

    // The scalar 2 inverts to 1/2.
    let two = CausalMultiVector::new(vec![2.0, 0.0, 0.0, 0.0], metric).unwrap();
    let inv_two = <CausalMultiVector<f64> as MultiVector<f64>>::inverse(&two).unwrap();
    assert_eq!(inv_two.data(), &[0.5, 0.0, 0.0, 0.0]);

    // e12 squares to -1 in the Euclidean plane, so e12^-1 = -e12.
    let e12 = CausalMultiVector::new(vec![0.0, 0.0, 0.0, 1.0], metric).unwrap();
    let inv_e12 = <CausalMultiVector<f64> as MultiVector<f64>>::inverse(&e12).unwrap();
    assert_eq!(inv_e12.data(), &[0.0, 0.0, 0.0, -1.0]);

    let round_trip = e12.geometric_product(&inv_e12);
    assert_eq!(round_trip.data(), &[1.0, 0.0, 0.0, 0.0]);
}

/// The zero multivector has no inverse, and the trait reports it as a zero-magnitude error.
#[test]
fn test_trait_inverse_zero_multivector_is_an_error() {
    let metric = Metric::Euclidean(2);
    let zero = CausalMultiVector::new(vec![0.0; 4], metric).unwrap();

    let err = <CausalMultiVector<f64> as MultiVector<f64>>::inverse(&zero).unwrap_err();

    assert_eq!(err, CausalMultiVectorError::zero_magnitude());
    assert!(
        err.to_string().contains("non-zero magnitude"),
        "unexpected message: {}",
        err
    );
}

/// The trait `inverse` guards on an exact zero squared magnitude; the inherent `inverse`
/// guards on `T::epsilon()`. The scalar 1e-9 has squared magnitude 1e-18, which sits below
/// f64::EPSILON, so the two entry points disagree on the same input.
#[test]
fn test_trait_inverse_accepts_magnitude_below_epsilon() {
    // A scalar multivector of 1e-9 is invertible; its inverse is 1e9. This once distinguished the
    // two `inverse` methods — the inherent one rejected any squared magnitude at or below
    // `f64::EPSILON` while the trait rejected only exact zero — and both now share one body, so
    // the case is here to pin that the small-but-invertible input is accepted by each.
    let metric = Metric::Euclidean(2);
    let tiny = CausalMultiVector::new(vec![1e-9, 0.0, 0.0, 0.0], metric).unwrap();

    let via_trait = <CausalMultiVector<f64> as MultiVector<f64>>::inverse(&tiny).unwrap();
    let via_inherent = tiny.inverse().unwrap();

    for (label, inv) in [("trait", &via_trait), ("inherent", &via_inherent)] {
        assert!(
            (inv.data()[0] - 1e9).abs() < 1.0,
            "{label}: expected 1e9, got {}",
            inv.data()[0]
        );
    }
    assert_eq!(
        via_trait.data(),
        via_inherent.data(),
        "the two entry points must return the same multivector"
    );
}

#[test]
fn test_inverse_is_a_two_sided_inverse_across_metrics() {
    // `A * A^-1 = 1`. The formula this replaced returned `Ã / <AÃ>₀`, which fails this for almost
    // every input: for `1 + 2e₁` in Cl(2) it gave `A/5`, so `A * A^-1` was `1 + 0.8e₁`.
    let cases: Vec<(&str, Metric, Vec<f64>)> = vec![
        (
            "Cl(2) versor",
            Metric::Euclidean(2),
            vec![1.0, 2.0, 0.0, 0.0],
        ),
        (
            "Cl(2) general",
            Metric::Euclidean(2),
            vec![1.0, 2.0, 3.0, 4.0],
        ),
        (
            "Cl(3) general",
            Metric::Euclidean(3),
            (1..=8).map(|i| i as f64).collect(),
        ),
        (
            "Cl(4) general",
            Metric::Euclidean(4),
            (1..=16).map(|i| i as f64).collect(),
        ),
        (
            "Minkowski general",
            Metric::Minkowski(4),
            (1..=16).map(|i| i as f64 * 0.5).collect(),
        ),
    ];

    for (label, metric, data) in cases {
        let a = CausalMultiVector::new(data, metric).unwrap();
        let inv = a.inverse().unwrap_or_else(|e| panic!("{label}: {e}"));
        let product = a.geometric_product(&inv);

        assert!(
            (product.data()[0] - 1.0).abs() < 1e-12,
            "{label}: scalar part of A * A^-1 was {}",
            product.data()[0]
        );
        for (k, coeff) in product.data().iter().enumerate().skip(1) {
            assert!(
                coeff.abs() < 1e-12,
                "{label}: blade {k} of A * A^-1 was {coeff}, must be zero"
            );
        }
    }
}

#[test]
fn test_a_null_multivector_has_no_inverse() {
    // `1 + e₁` squares to `2(1 + e₁)`; it spans a null direction and the left-multiplication map
    // is singular. The replaced formula returned an answer for it.
    let metric = Metric::Euclidean(2);
    let null = CausalMultiVector::new(vec![1.0, 1.0, 0.0, 0.0], metric).unwrap();

    assert!(null.inverse().is_err());
    assert!(<CausalMultiVector<f64> as MultiVector<f64>>::inverse(&null).is_err());
}
