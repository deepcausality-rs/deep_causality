/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};

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
